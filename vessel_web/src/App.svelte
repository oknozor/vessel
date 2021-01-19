<script>
    import Navbar from "./components/Navbar.svelte";
    import {RouterView} from '@bjornlu/svelte-router'
    import StatusBar from "./components/StatusBar.svelte";
</script>

<root>
    <Navbar/>
    <StatusBar/>
    <main>
        <div class="container-wrapper">
            <div class="container">
                <RouterView/>
            </div>
        </div>
    </main>
</root>

<style>
    main {
        padding-left: 250px;
    }

    .container-wrapper {
        display: flex;
        min-height: 924px;
    }

    .container {
        max-width: 840px;
        width: 100%;
        box-sizing: border-box;
        margin: 0px auto;
        padding: 0;
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
