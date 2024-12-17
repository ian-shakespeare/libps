package runes

import (
	"bufio"
	"errors"
	"io"
	"unicode/utf8"
)

type Reader struct {
	*bufio.Reader
}

func NewReader(r io.Reader) *Reader {
	return &Reader{bufio.NewReader(r)}
}

func (r *Reader) PeekRunes(n int) ([]rune, error) {
	if n < 1 {
		return nil, nil
	}

	word := []rune{}
	peekOffset := 0

	for i := 0; i < n; i++ {
	charBuilder:
		for peekBytes := 4; peekBytes > 0; peekBytes-- {
			b, err := r.Peek(peekBytes + peekOffset)
			if err != nil {
				continue charBuilder
			}

			char, _ := utf8.DecodeRune(b[peekOffset:])
			if char == utf8.RuneError {
				return nil, errors.New("rune error")
			}

			peekOffset += utf8.RuneLen(char)
			word = append(word, char)
			break charBuilder
		}
	}

	return word, nil
}

func (r *Reader) ReadRunes(n int) ([]rune, int, error) {
	word := []rune{}
	wordSize := 0

	for i := 0; i < n; i++ {
		char, charSize, err := r.ReadRune()
		if err != nil {
			return word, wordSize, err
		}

		word = append(word, char)
		wordSize += charSize
	}

	return word, wordSize, nil
}

func ToUTF8(rs []rune) []byte {
	return []byte(string(rs))
}
