export function rawToImageData(width: number, height: number, data: number[]): ImageData {
	// Create a new ImageData object
	const imageData = new ImageData(width, height);

	// Loop through each pixel in the data array
	for (let i = 0; i < data.length; i++) {
		// Get the grayscale value for this pixel
		const grayValue = data[i];

		// Calculate the index in the ImageData array
		const index = i * 4;

		// Set the RGBA values for this pixel
		imageData.data[index] = grayValue; // Red
		imageData.data[index + 1] = grayValue; // Green
		imageData.data[index + 2] = grayValue; // Blue
		imageData.data[index + 3] = 255; // Alpha (fully opaque)
	}

	return imageData;
}

export function removeAllChildren(node: Element) {
	while (node.firstChild) {
		if (node.lastChild) {
			node.removeChild(node.lastChild);
		}
	}
}

export function setChild(id: string, el: Element) {
	const node = document.getElementById(id);
	if (!node) return;

	removeAllChildren(node);
	node.appendChild(el);
}
