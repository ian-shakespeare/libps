package iterator

import "iter"

func Collect[T any](it iter.Seq[T]) []T {
	p := []T{}
	for value := range it {
		p = append(p, value)
	}
	return p
}

func Collect2[K, V any](it iter.Seq2[K, V]) ([]K, []V) {
	leftElems := []K{}
	rightElems := []V{}
	for left, right := range it {
		leftElems = append(leftElems, left)
		rightElems = append(rightElems, right)
	}
	return leftElems, rightElems
}
