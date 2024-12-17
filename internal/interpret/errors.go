package interpret

import "fmt"

type ScanError struct {
	Type    string
	Message string
}

func NewSyntaxError(message string) *ScanError {
	return &ScanError{
		Type:    "syntaxerror",
		Message: message,
	}
}

func NewSyntaxErrorf(format string, a ...any) *ScanError {
	return NewSyntaxError(fmt.Sprintf(format, a...))
}

func (s *ScanError) Error() string {
	return fmt.Sprintf("%s: %s", s.Type, s.Message)
}
