package main

import "C"
import "fmt"

//export GoFunction
func GoFunction() {
	fmt.Println("Hello from Go!")
}

//export AddNumbers
func AddNumbers(a, b int) int {
	return a + b
}

func main() {} // Required but unused
