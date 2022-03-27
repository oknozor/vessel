<script>
    import {afterUpdate} from "svelte";
    import SearchResultElement from "./SearchResultElement.svelte";


    export let results = []
    export let searchTicket = -1;
    const offset = 10;
    let start = 0;
    let end = 10;
    let loadedResults = []
    let loaded = false

    // Reset state on search ticket changed
    $: {
        start = 0;
        end = 10;
        loadedResults = []
        loaded = false
    }

    afterUpdate(() => {
        let unInit = results.length > offset;
        if (unInit) {
            loadResult();
            loaded = true
        }
    });

    window.addEventListener("scroll", () => {
        let isMaxScroll = window.innerHeight + window.scrollY >= document.body.offsetHeight
        if (isMaxScroll) {
            loadResult();
        }
    })

    const loadResult = () => {
        loadedResults = loadedResults.concat(...results.slice(start, end))
        start = end

        if (end + offset > results.length) {
            end = results.length
        } else {
            end = end + offset
        }
    }
</script>

<div class="flex flex-col">
    {#each loadedResults as result}
        <SearchResultElement {...result}/>
    {/each}
</div>
