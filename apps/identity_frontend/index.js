function userIdFromUrl() {
	const params = new URLSearchParams(window.location.search);
	return params.get("user_id");
}

async function createPasskey(challenge, user_id) {
	console.assert(user_id !== "");
	// From IANA COSE Algorithms registry
	const ED25519_ALG = -8;
	const publicKeyCredentialCreationOptions = {
	  challenge: challenge,
	  rp: { id: window.location.hostname, name: "Nexus Social" },
	  user: {
		id: new Uint8Array(user_id),
		name: user_id,
		displayName: user_id,
	  },
	  pubKeyCredParams: [{ type: "public-key", alg: ED25519_ALG }]
	};

	const credential = await navigator.credentials.create({ publicKey: publicKeyCredentialCreationOptions});
	console.log(credential);
}


function onFormSubmit(event) {
	console.log("login pressed");
	event.preventDefault();
	// const challenge = window.crypto.getRandomValues(new Uint8Array(8));
	const challenge = new Uint8Array(0);
	const user_id = userInputBox.value;
	createPasskey(challenge, user_id);
}
