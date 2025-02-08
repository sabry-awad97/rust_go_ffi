package main

import "C"
import (
	"fmt"
)

//export GetDLLVersion
func GetDLLVersion() C.longlong {
	// Version format: major * 10000 + minor * 100 + patch
	// For version 0.1.0 this returns 100
	return C.longlong(100) // represents 0.1.0
}

//export GoFunction
func GoFunction() {
	fmt.Println("Hello from Go!")
}

//export AddNumbers
func AddNumbers(a, b C.longlong) C.longlong {
	return a + b
}

func main() {} // Required but unused
