<script>
    import File from "./File.svelte";

    export let files = []

</script>

<div class="flex flex-col">
    <div class="-my-2  sm:-mx-6 lg:-mx-8">
        <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
            <div class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                    <tr>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Name
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Extension
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Size
                        </th>
                    </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200" x-max="1">
                    {#each files as file}
                        <File {...file}/>
                    {/each}
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</div>