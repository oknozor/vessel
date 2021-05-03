<script>
    export let name;
    export let extension;
    export let size;
    export let username;

    async function download() {
        await fetch(`http://localhost:3030/peers/${username}/queue`, {
            method: 'POST',
            body: JSON.stringify({
                filename: name
            }),
            headers: {
                'Content-Type': 'application/json',
                'Accept': '*/*'
            }
        });
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
        <button on:click|preventDefault={download}>Download</button>
    </td>
</tr>