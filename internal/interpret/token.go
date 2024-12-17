package interpret

type TokenType int

const (
	UNKNOWN_TOKEN       TokenType = 0
	INT_TOKEN           TokenType = 1
	REAL_TOKEN          TokenType = 2
	RADIX_TOKEN         TokenType = 3
	LIT_STRING_TOKEN    TokenType = 4
	HEX_STRING_TOKEN    TokenType = 5
	BASE85_STRING_TOKEN TokenType = 6
	NAME_TOKEN          TokenType = 7
)

type Token struct {
	Type  TokenType
	Value []rune
}

func (t *Token) Append(char ...rune) {
	t.Value = append(t.Value, char...)
}
