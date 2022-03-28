<script>

    export let name;
    export let extension;
    export let size;
    export let username;
    export let ticket;
    export let progress;

    async function download() {
        let body = JSON.stringify({
            file_name: name
        });

        console.log(body);

        await fetch(`http://localhost:3030/${username}/queue`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body
        })
            .then(response => {
                console.log(response);
            })
            .catch(err => {
                console.error(err);
            });    }
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
        {#if ticket}
            <p>{progress}</p>
        {:else}
            <button on:click|preventDefault={download}>Download</button>
        {/if}
    </td>
</tr>