package main

import (
	// "bufio"
	"fmt"
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

	tokens, err := interpret.Scan(strings.NewReader("(this is a\nmultline string)"))
	if err != nil {
		log.Fatal(err.Error())
	}

	fmt.Println(tokens)
}
