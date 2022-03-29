<script>

    import {createDownloadStore} from "../../createSearchStore";

    export let name;
    export let extension;
    export let size;
    export let username;
    export let ticket;
    let progress = 0;
    let started = false;

    const onDownload = () => {
        download();
        createDownloadStore().subscribe(value => {
            started = true;

            if (name.includes(value.file_name) && value.user_name === username) {
                ticket = value.ticket
            }

            if (ticket === value.ticket) {
                if (value.percent) {
                    progress = value.percent
                }
            }
        });
    };

    $: {
        let bar = document.getElementById(ticket);
        if (bar) {
            bar.style.width = progress + "%";
        }
    }

    async function download() {
        let body = JSON.stringify({
            file_name: name
        });

        await fetch(`http://localhost:3030/peers/${username}/queue`, {
            method: "POST",
            headers: {
                "content-Type": "application/json"
            },
            body
        })
    }
</script>

<tr>
    <td class="px-6 py-4 whitespace-nowrap">
        <div class="flex items-center">
            <div class="ml-4">
                <span class="text-sm font-medium text-gray-900">
                    {name}
                </span>
            </div>
        </div>
    </td>
    <td class="px-6 py-4 whitespace-nowrap">
        <span class="text-sm text-gray-900">{extension}</span>
    </td>
    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {size}
    </td>
    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {#if started}
            <div>
                <div class="w-full h-6 bg-gray-200 rounded-full dark:bg-gray-700">
                    <div id="{ticket}" class="h-6 bg-gray-600 rounded-full dark:bg-gray-300" style="width:{progress}%">{progress}%</div>
                </div>
            </div>
        {:else}
            <button on:click|preventDefault={onDownload}>Download</button>
        {/if}
    </td>
</tr>
