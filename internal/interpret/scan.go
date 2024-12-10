package interpret

import (
	"errors"
	"io"

	"github.com/ian-shakespeare/libps/pkg/utils"
)

func Scan(r io.Reader) ([]Token, error) {
	tokens := []Token{}

	buf, err := io.ReadAll(r)
	if err != nil {
		return nil, err
	}

	for i := 0; i < len(buf); {
		switch buf[i] {
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			i += 1
		case '%':
			newIndex, err := scanComment(buf, i)
			if err != nil {
				return nil, err
			}
			i = newIndex
		case '.', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			token, newIndex, err := scanNumeric(buf, i)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, token)
			i = newIndex
		case '(':
			token, newIndex, err := scanString(buf, i)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, token)
			i = newIndex
		default:
			token, newIndex, err := scanName(buf, i)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, token)
			i = newIndex
		}
	}

	return tokens, nil
}

func scanComment(buf []byte, startingIndex int) (int, error) {
	var i int
	for i = startingIndex + 1; i < len(buf); i++ {
		if buf[i] == '\n' || buf[i] == '\f' {
			break
		}
	}

	return i, nil
}

// TODO: Support scientific notation and radix numbers.
func scanNumeric(buf []byte, startingIndex int) (Token, int, error) {
	word := []byte{buf[startingIndex]}

	var i int
wordBuilder:
	for i = startingIndex + 1; i < len(buf); i++ {
		switch buf[i] {
		case '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9':
			word = append(word, buf[i])
		case '\x00', ' ', '\t', '\r', '\n', '\b', '\f':
			break wordBuilder
		default:
			return scanName(buf, startingIndex)
		}
	}

	t := INT_TOKEN
	hasDecimal := utils.Contains(word, '.')
	isRadix := utils.Contains(word, '#')
	if isRadix && hasDecimal {
		return Token{}, -1, errors.New("radix numeric may not contain a decimal mark")
	} else if hasDecimal {
		t = REAL_TOKEN
	}

	return Token{Type: t, Value: word}, i, nil
}

// TODO: Support \ddd
func scanString(buf []byte, startingIndex int) (Token, int, error) {
	word := []byte{}

	activeParens := 0

	var i int
wordBuilder:
	for i = startingIndex + 1; i < len(buf); i++ {
		switch buf[i] {
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
			if i+1 >= len(buf) {
				return Token{}, -1, errors.New("received unexpected EOF")
			}
			switch buf[i+1] {
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
				if i+2 < len(buf) && buf[i+2] == '\n' {
					i++
				}
			default:
			}
			i++
		default:
			word = append(word, buf[i])
		}
	}
	if i >= len(buf) {
		return Token{}, -1, errors.New("received unexpected EOF")
	}

	return Token{Type: STRING_TOKEN, Value: word}, i + 1, nil
}

func scanName(buf []byte, startingIndex int) (Token, int, error) {
	word := []byte{}

	var i int
	for i = startingIndex; i < len(buf); i++ {
		if utils.Contains([]byte{'\x00', ' ', '\t', '\r', '\n', '\b', '\f'}, buf[i]) {
			break
		}
		word = append(word, buf[i])
	}

	return Token{Type: NAME_TOKEN, Value: word}, i, nil
}
