package main

import (
	// "bufio"
	"errors"
	"fmt"
	"io"
	"log"

	// "os"
	"strings"

	"github.com/ian-shakespeare/libps/internal/interpret"
)

func main() {
	/*
		r := bufio.NewReader(os.Stdin)
		input, err := r.ReadString('\n')
		if err != nil {
			log.Fatal(err.Error())
		}
	*/

	scanner := interpret.NewScanner(strings.NewReader("(this is a string)"))

	tokens := []interpret.Token{}

	for token, err := range scanner.Tokens() {
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			log.Fatal(err.Error())
		}
		tokens = append(tokens, token)
	}

	fmt.Println(tokens)
}
