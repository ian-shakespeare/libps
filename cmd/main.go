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

	scanner := interpret.NewScanner(strings.NewReader(`
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
    `))

	tokens := []interpret.Token{}

	for {
		token, err := scanner.ReadToken()
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
