<script>
    import './main.css';
    import './GlobalStyle.svelte'
    import Navbar from "./Navbar.svelte";
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
                <header>
                    <h1>Logs</h1>
                </header>
                <ul></ul>
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

    header {
        display: flex;
        flex-direction: column;
        -moz-box-align: center;
        align-items: center;
        margin-bottom: 48px;
    }

    h1 {
        box-sizing: border-box;
        margin: 0px;
        min-width: 0px;
        color: #181922;
        font-size: 24px;
        font-weight: 400;
        line-height: 1.3;
        letter-spacing: -0.7px;
        font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol";
        padding: 0;
    }

    ul {
        display: flex;
        flex-wrap: wrap;
    }

    main {
        padding-left: 250px;
    }
</style>
