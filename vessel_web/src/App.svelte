<script>
	let eventsource = new EventSource("http://127.0.0.1:3031/events");

	eventsource.onmessage = event => {
		let eventList = document.querySelector('ul');
		let newElement = document.createElement("li");

		newElement.textContent = "message: " + event.data;
		eventList.appendChild(newElement);
	}
</script>

<main>
	<h1> Vessel Web </h1>
	<ul></ul>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>
