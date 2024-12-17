package runes_test

import (
	"fmt"
	"strings"
	"testing"

	"github.com/ian-shakespeare/libps/pkg/runes"
	"github.com/stretchr/testify/assert"
)

func TestPeekRunes(t *testing.T) {
	t.Parallel()

	input := []rune("this is a rune string with 41 characters!")

	for i := 1; i <= len(input); i++ {
		name := fmt.Sprintf("peek%dChar", i)
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			part := input[:i]
			s := strings.NewReader(string(input))
			r := runes.NewReader(s)

			char, err := r.PeekRunes(i)
			assert.NoError(t, err)
			assert.Equal(t, string(part), string(char))
		})
	}
}
