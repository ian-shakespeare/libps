package interpret

import (
	"encoding/ascii85"
	"errors"
	"io"
	"strconv"

	"github.com/ian-shakespeare/libps/pkg/array"
	"github.com/ian-shakespeare/libps/pkg/runes"
)

type scanner struct {
	reader *runes.Reader
}

func NewScanner(input io.Reader) *scanner {
	return &scanner{
		reader: runes.NewReader(input),
	}
}

func (s *scanner) ReadToken() (Token, error) {
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
			return s.ReadToken()
		case '.':
			token.Append(char)
			err = s.readReal(&token)
			return token, err
		case '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			token.Append(char)
			err = s.readNumeric(&token)
			return token, err
		case '(':
			err = s.readLiteralString(&token)
			return token, err
		case '<':
			next, err := s.reader.PeekRunes(1)
			if err != nil {
				return Token{}, err
			}

			if next[0] == '~' {
				_, _, err = s.reader.ReadRune()
				if err != nil {
					return Token{}, err
				}

				err = s.readBase85String(&token)
			} else {
				err = s.readHexString(&token)
			}
			return token, err
		default:
			token.Append(char)
			err = s.readName(&token)
			return token, err
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
				return NewSyntaxError("radix number cannot have a negative base")
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
				return NewSyntaxError("unexpected end of real number")
			}
			break
		}
		if err != nil {
			return err
		}

		switch char {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			if hasTrailingExponent {
				return NewSyntaxError("unexpected end of real number")
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
				return NewSyntaxError("unexpected end of radix number")
			}
			break
		}
		if err != nil {
			return err
		}

		switch char {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			if hasTrailingHash {
				return NewSyntaxError("unexpected end of radix number")
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

func (s *scanner) readLiteralString(token *Token) error {
	token.Type = LIT_STRING_TOKEN
	activeParens := 0

wordBuilder:
	for {
		char, _, err := s.reader.ReadRune()
		if errors.Is(err, io.EOF) {
			return NewSyntaxError("unexpected end of file")
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
			afterSlash, _, err := s.reader.ReadRune()
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
				continue wordBuilder
			case '\r':
				afterCrlf, err := s.reader.PeekRunes(1)
				if err != nil {
					return err
				}
				if afterCrlf[0] == '\n' {
					if _, _, err = s.reader.ReadRune(); err != nil {
						return err
					}
				}
			case '0', '1', '2', '3', '4', '5', '6', '7':
				octal := []rune{afterSlash}
				nextDigits, err := s.reader.PeekRunes(2)
				if err != nil {
					return nil
				}
				octal = append(octal, nextDigits...)
				value, err := strconv.ParseInt(string(octal), 8, 32)
				if err != nil {
					return NewSyntaxErrorf("unrecognized escape sequence: %s", string(octal))
				}
				if _, _, err = s.reader.ReadRunes(2); err != nil {
					return err
				}
				token.Append(rune(value))
			default:
				token.Append(afterSlash)
			}
		default:
			token.Append(char)
		}
	}

	return nil
}

func (s *scanner) readHexString(token *Token) error {
	token.Type = HEX_STRING_TOKEN

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
		case '>':
			if len(token.Value)&1 != 0 {
				token.Append('0')
			}
			break wordBuilder
		case '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F':
			token.Append(char)
		default:
			return NewSyntaxError("invalid hexidecimal")
		}
	}

	return nil
}

func (s *scanner) readBase85String(token *Token) error {
	token.Type = BASE85_STRING_TOKEN

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
		case '~':
			next, err := s.reader.PeekRunes(1)
			if err != nil {
				return err
			}

			if next[0] == '>' {
				if _, _, err = s.reader.ReadRune(); err != nil {
					return err
				}
				break wordBuilder
			}
			token.Append(char)
		default:
			token.Append(char)
		}
	}

	_, _, err := ascii85.Decode(nil, runes.ToUTF8(token.Value), true)
	if err != nil {
		return NewSyntaxError("invalid base85")
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
