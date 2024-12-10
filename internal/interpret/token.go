package interpret

type TokenType int

const (
	INT_TOKEN    TokenType = 0
	REAL_TOKEN   TokenType = 1
	STRING_TOKEN TokenType = 2
	NAME_TOKEN   TokenType = 3
)

type Token struct {
	Type  TokenType
	Value []byte
}
