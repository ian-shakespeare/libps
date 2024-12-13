package interpret

type TokenType int

const (
	INT_TOKEN    TokenType = 0
	REAL_TOKEN   TokenType = 1
	RADIX_TOKEN  TokenType = 2
	STRING_TOKEN TokenType = 3
	NAME_TOKEN   TokenType = 4
)

type Token struct {
	Type  TokenType
	Value []rune
}
