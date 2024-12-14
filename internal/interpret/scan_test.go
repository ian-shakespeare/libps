package interpret_test

import (
	"io"
	"strings"
	"testing"

	"github.com/ian-shakespeare/libps/internal/interpret"
	"github.com/ian-shakespeare/libps/pkg/array"
	"github.com/ian-shakespeare/libps/pkg/iterator"
	"github.com/stretchr/testify/assert"
)

func TestScan(t *testing.T) {
	t.Parallel()

	t.Run("comment", func(t *testing.T) {
		s := interpret.NewScanner(strings.NewReader("% this is a comment"))
		_, err := s.NextToken()
		assert.ErrorIs(t, io.EOF, err)
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

			s := interpret.NewScanner(strings.NewReader(input.value))
			token, err := s.NextToken()
			assert.NoError(t, err)
			assert.Equal(t, interpret.NAME_TOKEN, token.Type)
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
		{"realScientific", "1.2E7", interpret.REAL_TOKEN},
		{"realScientificLowerCase", "1.2e7", interpret.REAL_TOKEN},
		{"realScientificNegative", "-1.2e7", interpret.REAL_TOKEN},
		{"realScientificFraction", "1.2e-7", interpret.REAL_TOKEN},
		{"realScientificNegativeFraction", "-1.2e-7", interpret.REAL_TOKEN},
		{"radixBase2", "2#1000", interpret.RADIX_TOKEN},
		{"radixBase8", "8#1777", interpret.RADIX_TOKEN},
		{"radixBase16", "16#FFFE", interpret.RADIX_TOKEN},
	}

	for _, input := range validNumerics {
		t.Run(input.name, func(t *testing.T) {
			t.Parallel()

			s := interpret.NewScanner(strings.NewReader(input.value))
			token, err := s.NextToken()
			assert.NoError(t, err)
			assert.Equal(t, interpret.Token{Type: input.tokenType, Value: []rune(input.value)}, token)
		})
	}

	t.Run("stringUnterminated", func(t *testing.T) {
		t.Parallel()

		for _, input := range []string{"(this is a string", "(this is a string \\)"} {
			s := interpret.NewScanner(strings.NewReader(input))
			_, err := s.NextToken()
			assert.Error(t, err)
			assert.NotErrorIs(t, io.EOF, err)
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

			s := interpret.NewScanner(strings.NewReader(input.value))
			token, err := s.NextToken()
			assert.NoError(t, err)
			assert.Equal(t, input.expect, string(token.Value))
		})
	}

	escapedStrings := []struct {
		name   string
		value  string
		expect string
	}{
		{"stringNewline", "(\\n)", "\n"},
		{"stringCrlf", "(\\r)", "\r"},
		{"stringTab", "(\\t)", "\t"},
		{"stringBackspace", "(\\b)", "\b"},
		{"stringForm", "(\\f)", "\f"},
		{"stringSlash", "(\\\\)", "\\"},
		{"stringLParen", "(\\()", "("},
		{"stringRParen", "(\\))", ")"},
		{"stringIgnoreNewLine", "(\\\n)", ""},
		{"stringIgnoreCrlf", "(\\\r)", ""},
		{"stringIgnoreCrlfNewLine", "(\\\r\n)", ""},
	}

	for _, input := range escapedStrings {
		t.Run(input.name, func(t *testing.T) {
			t.Parallel()

			s := interpret.NewScanner(strings.NewReader(input.value))
			token, err := s.NextToken()
			assert.NoError(t, err)
			assert.Equal(t, []rune(input.expect), token.Value)
		})
	}

	t.Run("stringOctal", func(t *testing.T) {
		t.Parallel()

		s := interpret.NewScanner(strings.NewReader("(\\777)"))
		token, err := s.NextToken()
		assert.NoError(t, err)
		assert.Equal(t, 511, token.Value[0])
	})

	t.Run("name", func(t *testing.T) {
		t.Parallel()

		inputs := []string{"abc", "Offset", "$$", "23A", "13-456", "a.b", "$MyDict", "@pattern"}

		for _, input := range inputs {
			s := interpret.NewScanner(strings.NewReader(input))
			token, err := s.NextToken()
			assert.NoError(t, err)
			assert.Equal(t, string(input), string(token.Value))
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
			{Type: interpret.NAME_TOKEN, Value: []rune("myStr")},
			{Type: interpret.STRING_TOKEN, Value: []rune("i have a string right here")},
			{Type: interpret.NAME_TOKEN, Value: []rune("myOtherStr")},
			{Type: interpret.STRING_TOKEN, Value: []rune("and\nanother right here")},
			{Type: interpret.NAME_TOKEN, Value: []rune("myInt")},
			{Type: interpret.INT_TOKEN, Value: []rune("1234567890")},
			{Type: interpret.NAME_TOKEN, Value: []rune("myNegativeInt")},
			{Type: interpret.INT_TOKEN, Value: []rune("-1234567890")},
			{Type: interpret.NAME_TOKEN, Value: []rune("myReal")},
			{Type: interpret.REAL_TOKEN, Value: []rune("3.1456")},
			{Type: interpret.NAME_TOKEN, Value: []rune("myNegativeReal")},
			{Type: interpret.REAL_TOKEN, Value: []rune("-3.1456")},
		}

		s := interpret.NewScanner(strings.NewReader(input))
		tokens, errs := iterator.Collect2(s.Tokens())
		assert.False(t, array.Some(errs, func(err error) bool {
			return err != nil
		}))
		assert.Len(t, tokens, len(expect))
		assert.Equal(t, expect, tokens)

		// t.Log(tokens)
	})
}
