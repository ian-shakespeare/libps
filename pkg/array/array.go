package array

// Returns the index of the first element that is true on the condition.
// Otherwise, returns -1.
func Some[T any](arr []T, cond func(T) bool) int {
	for i := 0; i < len(arr); i++ {
		if cond(arr[i]) {
			return i
		}
	}
	return -1
}

// Returns true if the array contains the given value.
func Contains[T comparable](arr []T, value T) bool {
	index := Some(arr, func(elem T) bool {
		return elem == value
	})
	return index > -1
}
