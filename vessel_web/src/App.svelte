<script>
    import Navbar from "./components/Navbar.svelte";
    import {RouterView} from '@bjornlu/svelte-router'
    import StatusBar from "./components/StatusBar.svelte";
</script>

<root>
    <Navbar style="display: none" class="sm:hidden md:hidden"/>
    <StatusBar/>
    <main>
        <div class="container-wrapper">
            <div class="container">
                <RouterView/>
            </div>
        </div>
    </main>
</root>

<style global>
    @import url("https://cdn.jsdelivr.net/gh/jgthms/minireset.css@master/minireset.min.css");
    @import url('https://rsms.me/inter/inter.css');

    @tailwind base;
    @tailwind components;
    @tailwind utilities;

    body {
        font-family: "Inter", serif;
    }

    a:link, a:visited {
        text-decoration: none
    }

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
