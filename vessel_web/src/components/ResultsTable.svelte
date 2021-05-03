<script>
    import ResultsElement from "./ResultsElement.svelte";
    import {afterUpdate, onMount} from "svelte";


    export let results = []

    const offset = 10;
    let start = 0;
    let end = 10;

    let loadedResults = []
    let loaded = false

    afterUpdate(() => {
        let unInit = results.length > offset && !loaded;
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
    <div class="-my-2  sm:-mx-6 lg:-mx-8">
        <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
            <div class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                    <tr>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Ticket
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Username
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Files
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Slot free
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Queue length
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Average speed
                        </th>
                        <th scope="col"
                            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                            Locked results
                        </th>
                        <th scope="col" class="relative px-6 py-3">
                            <span class="sr-only">Edit</span>
                        </th>
                    </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200" x-max="1">
                    {#each loadedResults as result}
                        <ResultsElement {...result}/>
                    {/each}
                </table>
            </div>
        </div>
    </div>
</div>