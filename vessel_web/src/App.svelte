<script>
    import './GlobalStyle.svelte'
    import Navbar from "./components/Navbar.svelte";
    import { RouterView } from '@bjornlu/svelte-router'

    let eventsource = new EventSource("http://127.0.0.1:3031/events");

    eventsource.onmessage = event => {
        let eventList = document.querySelector('ul');
        let newElement = document.createElement("li");

        newElement.textContent = "message: " + event.data;
        eventList.appendChild(newElement);
    }
</script>

<root>
    <Navbar/>
    <main>
        <div style="display: flex; min-height: 924px;">
            <div class="container">
                <RouterView/>
            </div>

        </div>
    </main>
</root>

<style>
    @import url('https://rsms.me/inter/inter.css');

    main {
        padding-left: 250px;
    }

    .container {
        max-width: 840px;
        width: 100%;
        box-sizing: border-box;
        margin: 0px auto;
        padding: 48px 0px;
        display: flex;
        flex-direction: column;
    }



    ul {
        display: flex;
        flex-wrap: wrap;
    }

    main {
        padding-left: 250px;
    }
</style>
