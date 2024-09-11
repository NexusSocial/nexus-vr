function userIdFromUrl() {
	const params = new URLSearchParams(window.location.search);
	return params.get("user_id");
}

/**
 * @param {SubmitEvent} event
*/
async function onFormSubmit(event) {
	console.debug("form submitted");
	event.preventDefault();

	const userInputBox = document.getElementById("userId");
	const userId = userInputBox.value;

	const challenge = window.crypto.getRandomValues(new Uint8Array(8));
	// const challenge = new Uint8Array(0);

	let credential;
	if (event.submitter.id == "createButton") {
		console.debug("create");
		credential = await createPasskey(challenge, userId);
	} else if (event.submitter.id == "signButton") {
		console.debug("sign");
		credential = await signWithPasskey(challenge, userId);
	} else {
	  throw new Error("unknown submitter id");
	}
	console.debug("credential: ", credential);
}

async function createPasskey(challenge, userId) {
	console.assert(userId !== "");
	// From IANA COSE Algorithms registry
	const ED25519_ALG = -8;
	const ES256_ALG = -7;
	const publicKey = {
		challenge: challenge,
		rp: { id: window.location.hostname, name: "Nexus Social" },
		user: {
			id: (new TextEncoder()).encode(userId),
			name: userId,
			displayName: userId,
		},
		pubKeyCredParams: [],
	};
	const credential = await navigator.credentials.create({ publicKey });
	return credential;
}


async function signWithPasskey(challenge, userId) {
	const publicKey = {
		challenge: challenge,
		rpId: window.location.hostname,
		 allowCredentials: [{
		  type: "public-key",      
		  id: (new TextEncoder()).encode(userId),
		}],
		// see note below
		// userVerification: "preferred", 
	};
	return await navigator.credentials.get({ publicKey });
}
