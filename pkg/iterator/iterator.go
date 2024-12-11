package iterator

import "iter"

func Collect[T any](it iter.Seq[T]) []T {
	p := []T{}
	for value := range it {
		p = append(p, value)
	}
	return p
}

func Collect2[Left, Right any](it iter.Seq2[Left, Right]) ([]Left, []Right) {
	leftElems := []Left{}
	rightElems := []Right{}
	for left, right := range it {
		leftElems = append(leftElems, left)
		rightElems = append(rightElems, right)
	}
	return leftElems, rightElems
}
