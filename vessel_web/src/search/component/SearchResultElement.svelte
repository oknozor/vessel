<script>
    import FileTable from "./FileTable.svelte";
    export let ticket;
    export let username;
    export let files = [];
    export let slot_free;
    export let average_speed;
    export let queue_length;
    export let locked_results;
</script>

<div>
    <div>
        {username}
    </div>
    <div><FileTable {files} {username}/></div>
</div>
