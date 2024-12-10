package interpret_test

import (
	"strings"
	"testing"

	"github.com/ian-shakespeare/libps/internal/interpret"
	"github.com/stretchr/testify/assert"
)

func TestScan(t *testing.T) {
	t.Parallel()

	t.Run("comment", func(t *testing.T) {
		tokens, err := interpret.Scan(strings.NewReader("% this is a comment"))
		assert.NoError(t, err)
		assert.Empty(t, tokens)
	})

	invalidNumerics := []struct {
		name  string
		value string
	}{
		{"integerInvalid", "1x0"},
		{"realInvalid", "1.x0"},
	}

	for _, input := range invalidNumerics {
		t.Run(input.name, func(t *testing.T) {
			t.Parallel()

			tokens, err := interpret.Scan(strings.NewReader(input.value))
			assert.NoError(t, err)
			assert.Len(t, tokens, 1)
			assert.Equal(t, interpret.NAME_TOKEN, tokens[0].Type)
		})
	}

	validNumerics := []struct {
		name      string
		value     string
		tokenType interpret.TokenType
	}{
		{"integer", "1", interpret.INT_TOKEN},
		{"integerNegative", "-1", interpret.INT_TOKEN},
		{"integerMultidigit", "1234567890", interpret.INT_TOKEN},
		{"real", ".1", interpret.REAL_TOKEN},
		{"realNegative", "-.1", interpret.REAL_TOKEN},
		{"realMultidigit", "1.234567890", interpret.REAL_TOKEN},
	}

	for _, input := range validNumerics {
		t.Run(input.name, func(t *testing.T) {
			t.Parallel()

			r := strings.NewReader(input.value)

			tokens, err := interpret.Scan(r)
			assert.NoError(t, err)
			assert.Len(t, tokens, 1)
			assert.Equal(t, interpret.Token{Type: input.tokenType, Value: []byte(input.value)}, tokens[0])
		})
	}

	t.Run("stringUnterminated", func(t *testing.T) {
		t.Parallel()

		for _, input := range []string{"(this is a string", "(this is a string \\)"} {
			r := strings.NewReader(input)
			_, err := interpret.Scan(r)
			assert.Error(t, err)
		}
	})

	validStrings := []struct {
		name   string
		value  string
		expect string
	}{
		{"string", "(this is a string)", "this is a string"},
		{"stringMultiline", "(this is a multiline\nstring)", "this is a multiline\nstring"},
		{"stringMultilineWhitespace", "(this is a multiline\r\nstring)", "this is a multiline\r\nstring"},
	}

	for _, input := range validStrings {
		t.Run(input.name, func(t *testing.T) {
			t.Parallel()

			r := strings.NewReader(input.value)

			tokens, err := interpret.Scan(r)
			assert.NoError(t, err)
			assert.Len(t, tokens, 1)
			assert.Equal(t, input.expect, string(tokens[0].Value))
		})
	}

	t.Run("name", func(t *testing.T) {
		t.Parallel()

		inputs := []string{"abc", "Offset", "$$", "23A", "13-456", "a.b", "$MyDict", "@pattern"}

		for _, input := range inputs {
			r := strings.NewReader(input)

			tokens, err := interpret.Scan(r)
			assert.NoError(t, err)
			assert.Len(t, tokens, 1)
			assert.Equal(t, string(input), string(tokens[0].Value))
		}
	})

	t.Run("all", func(t *testing.T) {
		t.Parallel()

		input := `
myStr (i have a string right here)
myOtherStr (and
another \
right \
here)
% this is a comment
myInt 1234567890
myNegativeInt -1234567890
myReal 3.1456
myNegativeReal -3.1456
    `

		expect := []interpret.Token{
			{Type: interpret.NAME_TOKEN, Value: []byte("myStr")},
			{Type: interpret.STRING_TOKEN, Value: []byte("i have a string right here")},
			{Type: interpret.NAME_TOKEN, Value: []byte("myOtherStr")},
			{Type: interpret.STRING_TOKEN, Value: []byte("and\nanother right here")},
			{Type: interpret.NAME_TOKEN, Value: []byte("myInt")},
			{Type: interpret.INT_TOKEN, Value: []byte("1234567890")},
			{Type: interpret.NAME_TOKEN, Value: []byte("myNegativeInt")},
			{Type: interpret.INT_TOKEN, Value: []byte("-1234567890")},
			{Type: interpret.NAME_TOKEN, Value: []byte("myReal")},
			{Type: interpret.REAL_TOKEN, Value: []byte("3.1456")},
			{Type: interpret.NAME_TOKEN, Value: []byte("myNegativeReal")},
			{Type: interpret.REAL_TOKEN, Value: []byte("-3.1456")},
		}

		tokens, err := interpret.Scan(strings.NewReader(input))
		assert.NoError(t, err)
		assert.Len(t, tokens, len(expect))
		assert.Equal(t, expect, tokens)

		// t.Log(tokens)
	})
}
