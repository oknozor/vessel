<script>
    export let name;
    export let connected_users;
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
        <span class="text-sm text-gray-900">{connected_users}</span>
    </td>
</tr>