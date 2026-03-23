package main

import (
	"fmt"
	"os"
	"runtime"
	"time"
	"unsafe"
)

//go:wasmimport nats publish
func natsPublish(subjectPtr *byte, subjectLen uint32, payloadPtr *byte, payloadLen uint32)

func publish(subject, payload string) {
	subjectPtr, subjectLen := stringPtrLen(subject)
	payloadPtr, payloadLen := stringPtrLen(payload)

	natsPublish(subjectPtr, subjectLen, payloadPtr, payloadLen)

	// Defensive: keep strings alive through the import call.
	runtime.KeepAlive(subject)
	runtime.KeepAlive(payload)
}

func stringPtrLen(s string) (*byte, uint32) {
	if len(s) == 0 {
		return nil, 0
	}
	return unsafe.StringData(s), uint32(len(s))
}

func main() {
	name := os.Getenv("NAME")
	fmt.Printf("Hello, world! My name is %s\n", name)

	publish("hello", "Hello from Go!")
	time.Sleep(1 * time.Second)
	publish("goodbye", "Goodbye from Go!")

	fmt.Printf("Goodbye from %s\n", name)
}
