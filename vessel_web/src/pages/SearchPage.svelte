<script>
    import ResultsTable from "../components/ResultsTable.svelte";
    import {afterUpdate, onMount} from "svelte";
    import {createDownloadProgressStore, createDownloadStore, createSearchStore} from "../createSearchStore";

    let searchStore
    let downloadStore
    let downloadProgressStore

    let searchTerm = '';
    let searchResults = [];

    let searchTicket;

    onMount(() => {
        searchStore = createSearchStore();
        downloadStore = createDownloadStore();
        downloadProgressStore = createDownloadProgressStore();

        searchStore.subscribe(incomingMessages => {
            // Ensure we get only search response for the current ticket
            searchResults = incomingMessages.filter(message => {
                console.debug("got search result with ticket :" + message.ticket);
                return message.ticket === searchTicket
            });
        });

        downloadStore.subscribe(downloadStarting => {
            console.debug("Download started : ")
            console.debug(downloadStarting)
            let searchResultStartingDownload = searchResults.find(item => item.username === downloadStarting.user_name);

            if (searchResultStartingDownload) {
                searchResultStartingDownload.files.find(entry => entry.name === downloadStarting.file_name).ticket = downloadStarting.ticket
                console.debug("Found matching search result: ")
                console.debug(searchResultStartingDownload.files.find(entry => entry.name === downloadStarting.file_name));
                searchResults = [... searchResults]
            }
        });

        downloadProgressStore.subscribe(incomingMessages => {
            let file = searchResults.map(userFiles => userFiles.files)
                .flat()
                .find(file => file.ticket === incomingMessages.ticket);

            if (file) {
                file.progress = incomingMessages.percent;
                console.debug("Download progress ")
                console.debug(file)
                searchResults = [... searchResults]
            }
        });

        return () => {
            if (searchStore.readyState === 1) {
                console.debug("Closing event source")
                createSearchStore.close();
            }
        };
    });

    async function doSearch() {
        console.debug("Issuing search request : \"" + searchTerm + "\"");
        let response = await fetch(`http://localhost:3030/search?term=${searchTerm}`);
        if (response.ok) { // if HTTP-status is 200-299
            let json = await response.json();
            searchResults = []
            searchTicket = json.ticket
            console.debug("Now rendering search result for ticket : " + json.ticket)
        } else {
            alert("HTTP-Error: " + response.status);
        }
    }
</script>

<div class="w-full sticky top-0 flex items-center pt-8 pb-16">
    <form on:submit|preventDefault={doSearch} class="flex-1">
        <figure class="absolute flex items-center pl-3 pt-2">
            <svg height="100%" viewBox="0 0 28 28" width="28" class="css-w9q8es">
                <path fill-rule="evenodd" clip-rule="evenodd"
                      d="M21.0819 22.6659C21.5206 23.1114 22.2316 23.1114 22.6704 22.6659C23.1091 22.2215 23.1091 21.5026 22.6704 21.0571L19.2054 17.5921C20.1841 16.2781 20.7646 14.6491 20.7646 12.8829C20.7646 8.52913 17.2355 5 12.8829 5C8.52913 5 5 8.52913 5 12.8829C5 17.2366 8.52913 20.7657 12.8829 20.7657C14.6547 20.7657 16.2916 20.1807 17.6079 19.1941L21.0819 22.6659ZM7.25225 12.8829C7.25225 9.77338 9.77225 7.25225 12.8829 7.25225C15.9924 7.25225 18.5124 9.77225 18.5124 12.8829C18.5124 15.9924 15.9924 18.5135 12.884 18.5135C9.7745 18.5135 7.25338 15.9935 7.25338 12.8829H7.25225Z"></path>
            </svg>
        </figure>
        <input name="searchInput" placeholder="Search for music, or something else..." type="search"
               class="search-input" bind:value={searchTerm}>
    </form>
    <button class="wishlist-btn">Wishlist</button>
</div>

<ResultsTable results={searchResults} searchTicket={searchTicket}/>

<style>
    .search-input {
        box-sizing: border-box;
        margin: 0;
        min-width: 0;
        display: block;
        width: 100%;
        appearance: none;
        border: medium none;
        background-color: #E8E8E9;
        color: #181922;
        height: 44px;
        line-height: 44px;
        padding: 0 44px 0 58px;
        outline: currentcolor none medium;
        border-radius: 99px;
        transition: all 300ms cubic-bezier(0.23, 1, 0.32, 1) 0s;
        pointer-events: all;
        font-size: 14px;
    }

    .wishlist-btn {
        box-sizing: border-box;
        margin: 0 0 0 8px;
        min-width: 0;
        appearance: none;
        display: inline-block;
        text-align: center;
        text-decoration: none;
        border: 0 none;
        background-color: transparent;
        color: #131314;
        box-shadow: rgba(19, 19, 20, 0.22) 0 0 0 1px, rgba(0, 0, 0, 0.25) 0 4px 4px -5px;
        cursor: pointer;
        font-weight: 500;
        outline: currentcolor none medium;
        transition: all 300ms cubic-bezier(0.23, 1, 0.32, 1) 0s;
        padding: 0 24px;
        pointer-events: all;
        font-size: 14px;
        border-radius: 99px;
        height: 42px;
        line-height: 42px;
        position: relative;
    }
</style>