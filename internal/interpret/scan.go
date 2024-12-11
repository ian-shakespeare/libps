package interpret

import (
	"errors"
	"io"
	"iter"

	"github.com/ian-shakespeare/libps/pkg/array"
)

type scanner struct {
	input io.ReadSeeker
}

func NewScanner(input io.ReadSeeker) *scanner {
	return &scanner{
		input,
	}
}

func (s *scanner) NextToken() (Token, error) {
	for {
		b, err := s.getNextCharacter()
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
		case '.', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			token, err := s.scanNumeric(b)
			return token, err
		case '(':
			token, err := s.scanString()
			return token, err
		default:
			token, err := s.scanName(b)
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

func (s *scanner) getNextCharacter() (byte, error) {
	b := make([]byte, 1)
	if _, err := s.input.Read(b); err != nil {
		return 0, err
	}
	return b[0], nil
}

func (s *scanner) scanComment() error {
	for {
		b, err := s.getNextCharacter()
		if err != nil {
			return err
		}

		if b == '\n' || b == '\f' {
			break
		}
	}

	return nil
}

// TODO: Support scientific notation and radix numbers.
func (s *scanner) scanNumeric(startingChars ...byte) (Token, error) {
	word := startingChars

wordBuilder:
	for {
		b, err := s.getNextCharacter()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return Token{}, err
		}

		switch b {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			break wordBuilder
		case '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			word = append(word, b)
		default:
			word = append(word, b)
			return s.scanName(word...)
		}
	}

	t := INT_TOKEN
	hasDecimal := array.Contains(word, '.')
	isRadix := array.Contains(word, '#')
	if isRadix && hasDecimal {
		return Token{}, errors.New("radix numeric may not contain a decimal mark")
	} else if hasDecimal {
		t = REAL_TOKEN
	}

	return Token{Type: t, Value: word}, nil
}

// TODO: Support \ddd
func (s *scanner) scanString(startingChars ...byte) (Token, error) {
	word := startingChars

	activeParens := 0

wordBuilder:
	for {
		b, err := s.getNextCharacter()
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
			afterSlash, err := s.getNextCharacter()
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
				afterCrlf, err := s.getNextCharacter()
				if err != nil {
					return Token{}, err
				}
				if afterCrlf != '\n' {
					if _, err := s.input.Seek(-1, io.SeekCurrent); err != nil {
						return Token{}, err
					}
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
		b, err := s.getNextCharacter()
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
