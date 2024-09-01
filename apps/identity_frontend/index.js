function userIdFromUrl() {
	const params = new URLSearchParams(window.location.search);
	return params.get("user_id");
}

function onLoginPressed() {
	console.log("button pressed")

}
