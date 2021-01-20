<script>
    import ResultsElement from "./ResultsElement.svelte";

    //TODO: replace placeholder data
    export let results = [
        {
            username: 'johnswing',
            speed: '50 MiB/s',
            position: '0',
            directory: 'Stone Witcher/complete'
        },
        {
            username: 'okno',
            speed: '14 MiB/s',
            position: '12',
            directory: 'albums/pouet'
        },
        {
            username: 'the_gamble',
            speed: '500 MiB/s',
            position: '125',
            directory: 'bad_quality_music/rock/folder/other'
        },
        {
            username: 'chlgreen23',
            speed: '53 MiB/s',
            position: '0',
            directory: 'Stone Witcher/complete'
        },
        {
            username: 'johnswing',
            speed: '50 MiB/s',
            position: '0',
            directory: 'Stone Witcher/complete'
        },
        {
            username: 'okno',
            speed: '14 MiB/s',
            position: '12',
            directory: 'albums/pouet'
        },
        {
            username: 'the_gamble',
            speed: '500 MiB/s',
            position: '125',
            directory: 'bad_quality_music/rock/folder/other'
        },
        {
            username: 'the_gamble',
            speed: '500 MiB/s',
            position: '125',
            directory: 'bad_quality_music/rock/folder/other'
        }
    ]
</script>

<div class="flex flex-col">
    <div class="-my-2  sm:-mx-6 lg:-mx-8">
        <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
            <div class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                    <tr>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Name
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Directory
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Speed
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Queue
                        </th>
                        <th scope="col" class="relative px-6 py-3">
                            <span class="sr-only">Edit</span>
                        </th>
                    </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200" x-max="1">
                    {#each results as result}
                        <ResultsElement {...result}/>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</div>