<script>
    import FileTable from "./FileTable.svelte";
    export let ticket;
    export let username;
    export let files;
    export let slot_free;
    export let average_speed;
    export let queue_length;
    export let locked_results;
</script>

<tr>
    <td class="px-6 py-4 whitespace-nowrap">
        <div class="flex items-center">
            <div class="ml-4">
                <span class="text-sm font-medium text-gray-900">
                    {ticket}
                </span>
            </div>
        </div>
    </td>
    <td class="px-6 py-4 whitespace-nowrap">
        <span class="text-sm text-gray-900">{username}</span>
    </td>
    <td class="px-6 py-4 whitespace-nowrap">
                    <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                        <FileTable {files}/>
                    </span>
    </td>
    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {slot_free}
    </td>

    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {average_speed}
    </td>
    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {queue_length}
    </td>

    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {locked_results}
    </td>
</tr>