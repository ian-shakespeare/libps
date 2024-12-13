package interpret

import (
	"bufio"
	"errors"
	"io"
	"iter"

	"github.com/ian-shakespeare/libps/pkg/array"
)

type scanner struct {
	reader *bufio.Reader
}

func NewScanner(input io.Reader) *scanner {
	return &scanner{
		reader: bufio.NewReader(input),
	}
}

func (s *scanner) NextToken() (Token, error) {
	for {
		b, err := s.reader.ReadByte()
		if err != nil {
			return Token{}, err
		}
		switch b {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			continue
		case '%':
			if err := s.scanComment(); err != nil {
				return Token{}, err
			}
			return s.NextToken()
		case '.':
			return s.scanReal(b)
		case '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			return s.scanNumeric(b)
		case '(':
			return s.scanString()
		default:
			return s.scanName(b)
		}
	}
}

func (s *scanner) Tokens() iter.Seq2[Token, error] {
	return func(yield func(Token, error) bool) {
		for {
			token, err := s.NextToken()
			if errors.Is(err, io.EOF) {
				break
			}
			if !yield(token, err) {
				return
			}
		}
	}
}

func (s *scanner) scanComment() error {
	for {
		b, err := s.reader.ReadByte()
		if err != nil {
			return err
		}

		if b == '\n' || b == '\f' {
			break
		}
	}

	return nil
}

func (s *scanner) scanNumeric(startingChars ...byte) (Token, error) {
	word := startingChars

wordBuilder:
	for {
		b, err := s.reader.ReadByte()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return Token{}, err
		}

		switch b {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			break wordBuilder
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			word = append(word, b)
		case '.':
			word = append(word, b)
			return s.scanReal(word...)
		case '#':
			if word[0] == '-' {
				return Token{}, errors.New("radix number cannot have a negative base")
			}
			word = append(word, b)
			return s.scanRadix(word...)
		default:
			word = append(word, b)
			return s.scanName(word...)
		}
	}

	return Token{Type: INT_TOKEN, Value: word}, nil
}

func (s *scanner) scanReal(startingChars ...byte) (Token, error) {
	word := startingChars

wordBuilder:
	for {
		hasTrailingExponent := word[len(word)-1] == 'e' || word[len(word)-1] == 'E'

		b, err := s.reader.ReadByte()
		if errors.Is(err, io.EOF) {
			if hasTrailingExponent {
				return Token{}, errors.New("received unexpected end of real number")
			}
			break
		}
		if err != nil {
			return Token{}, err
		}

		switch b {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			if hasTrailingExponent {
				return Token{}, errors.New("received unexpected end of real number")
			}
			break wordBuilder
		case 'e', 'E':
			if array.Contains(word, 'e') || array.Contains(word, 'E') {
				return s.scanName(word...)
			}
			word = append(word, b)
		case '-':
			if !hasTrailingExponent {
				return s.scanName(word...)
			}
			word = append(word, b)
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			word = append(word, b)
		default:
			word = append(word, b)
			return s.scanName(word...)
		}
	}

	return Token{Type: REAL_TOKEN, Value: word}, nil
}

func (s *scanner) scanRadix(startingChars ...byte) (Token, error) {
	word := startingChars

wordBuilder:
	for {
		hasTrailingHash := word[len(word)-1] == '#'

		b, err := s.reader.ReadByte()
		if errors.Is(err, io.EOF) {
			if hasTrailingHash {
				return Token{}, errors.New("received unexpected end of radix number")
			}
			break
		}
		if err != nil {
			return Token{}, err
		}

		switch b {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			if hasTrailingHash {
				return Token{}, errors.New("received unexpected end of radix number")
			}
			break wordBuilder
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z':
			word = append(word, b)
		default:
			word = append(word, b)
			return s.scanName(word...)
		}
	}

	return Token{Type: RADIX_TOKEN, Value: word}, nil
}

// TODO: Support \ddd
func (s *scanner) scanString(startingChars ...byte) (Token, error) {
	word := startingChars

	activeParens := 0

wordBuilder:
	for {
		b, err := s.reader.ReadByte()
		if errors.Is(err, io.EOF) {
			return Token{}, errors.New("received unexpected end of file")
		}
		if err != nil {
			return Token{}, err
		}

		switch b {
		case '(':
			word = append(word, '(')
			activeParens++
		case ')':
			if activeParens < 1 {
				break wordBuilder
			}
			word = append(word, ')')
			activeParens--
		case '\\':
			afterSlash, err := s.reader.ReadByte()
			if err != nil {
				return Token{}, err
			}
			switch afterSlash {
			case 'n':
				word = append(word, '\n')
			case 'r':
				word = append(word, '\r')
			case 't':
				word = append(word, '\t')
			case 'b':
				word = append(word, '\b')
			case 'f':
				word = append(word, '\f')
			case '\\':
				word = append(word, '\\')
			case '(':
				word = append(word, '(')
			case ')':
				word = append(word, ')')
			case '\n':
			case '\r':
				afterCrlf, err := s.reader.Peek(1)
				if err != nil {
					return Token{}, err
				}
				if afterCrlf[0] == '\n' {
					_, _ = s.reader.ReadByte()
				}
			default:
				break
			}
		default:
			word = append(word, b)
		}
	}

	return Token{Type: STRING_TOKEN, Value: word}, nil
}

func (s *scanner) scanName(startingChars ...byte) (Token, error) {
	word := startingChars

	for {
		b, err := s.reader.ReadByte()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return Token{}, err
		}

		if array.Contains([]byte{'\x00', ' ', '\t', '\r', '\n', '\b', '\f'}, b) {
			break
		}
		word = append(word, b)
	}

	return Token{Type: NAME_TOKEN, Value: word}, nil
}
