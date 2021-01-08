<script>
    import './GlobalStyle.svelte'
    import Navbar from "./components/Navbar.svelte";
    import { RouterView } from '@bjornlu/svelte-router'
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
