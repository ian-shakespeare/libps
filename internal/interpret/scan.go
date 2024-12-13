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
	token := Token{Type: UNKNOWN_TOKEN, Value: []rune{}}

	for {
		char, _, err := s.reader.ReadRune()
		if err != nil {
			return Token{}, err
		}
		switch char {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			continue
		case '%':
			if err := s.readComment(); err != nil {
				return Token{}, err
			}
			return s.NextToken()
		case '.':
			token.Append(char)
			err = s.readReal(&token)
			return token, err
		case '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			token.Append(char)
			err = s.readNumeric(&token)
			return token, err
		case '(':
			err = s.readString(&token)
			return token, err
		default:
			token.Append(char)
			err = s.readName(&token)
			return token, err
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

func (s *scanner) readComment() error {
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

func (s *scanner) readNumeric(token *Token) error {
	token.Type = INT_TOKEN

wordBuilder:
	for {
		char, _, err := s.reader.ReadRune()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return err
		}

		switch char {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			break wordBuilder
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			token.Append(char)
		case '.':
			token.Append(char)
			return s.readReal(token)
		case '#':
			if token.Value[0] == '-' {
				return errors.New("radix number cannot have a negative base")
			}
			token.Append(char)
			return s.readRadix(token)
		default:
			token.Append(char)
			return s.readName(token)
		}
	}

	return nil
}

func (s *scanner) readReal(token *Token) error {
	token.Type = REAL_TOKEN

wordBuilder:
	for {
		hasTrailingExponent := token.Value[len(token.Value)-1] == 'e' || token.Value[len(token.Value)-1] == 'E'

		char, _, err := s.reader.ReadRune()
		if errors.Is(err, io.EOF) {
			if hasTrailingExponent {
				return errors.New("received unexpected end of real number")
			}
			break
		}
		if err != nil {
			return err
		}

		switch char {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			if hasTrailingExponent {
				return errors.New("received unexpected end of real number")
			}
			break wordBuilder
		case 'e', 'E':
			if array.Contains(token.Value, 'e') || array.Contains(token.Value, 'E') {
				return s.readName(token)
			}
			token.Append(char)
		case '-':
			if !hasTrailingExponent {
				return s.readName(token)
			}
			token.Append(char)
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			token.Append(char)
		default:
			token.Append(char)
			return s.readName(token)
		}
	}

	return nil
}

func (s *scanner) readRadix(token *Token) error {
	token.Type = RADIX_TOKEN

wordBuilder:
	for {
		hasTrailingHash := token.Value[len(token.Value)-1] == '#'

		char, _, err := s.reader.ReadRune()
		if errors.Is(err, io.EOF) {
			if hasTrailingHash {
				return errors.New("received unexpected end of radix number")
			}
			break
		}
		if err != nil {
			return err
		}

		switch char {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			if hasTrailingHash {
				return errors.New("received unexpected end of radix number")
			}
			break wordBuilder
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z':
			token.Append(char)
		default:
			token.Append(char)
			return s.readName(token)
		}
	}

	return nil
}

// TODO: Support \ddd
func (s *scanner) readString(token *Token) error {
	token.Type = STRING_TOKEN
	activeParens := 0

wordBuilder:
	for {
		char, _, err := s.reader.ReadRune()
		if errors.Is(err, io.EOF) {
			return errors.New("received unexpected end of file")
		}
		if err != nil {
			return err
		}

		switch char {
		case '(':
			token.Append('(')
			activeParens++
		case ')':
			if activeParens < 1 {
				break wordBuilder
			}
			token.Append(')')
			activeParens--
		case '\\':
			afterSlash, err := s.reader.ReadByte()
			if err != nil {
				return err
			}
			switch afterSlash {
			case 'n':
				token.Append('\n')
			case 'r':
				token.Append('\r')
			case 't':
				token.Append('\t')
			case 'b':
				token.Append('\b')
			case 'f':
				token.Append('\f')
			case '\\':
				token.Append('\\')
			case '(':
				token.Append('(')
			case ')':
				token.Append(')')
			case '\n':
			case '\r':
				afterCrlf, err := s.reader.Peek(1)
				if err != nil {
					return err
				}
				if afterCrlf[0] == '\n' {
					_, _ = s.reader.ReadByte()
				}
			default:
				break
			}
		default:
			token.Append(char)
		}
	}

	return nil
}

func (s *scanner) readName(token *Token) error {
	token.Type = NAME_TOKEN

	for {
		char, _, err := s.reader.ReadRune()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return err
		}

		if array.Contains([]rune{'\x00', ' ', '\t', '\r', '\n', '\b', '\f'}, char) {
			break
		}
		token.Append(char)
	}

	return nil
}
