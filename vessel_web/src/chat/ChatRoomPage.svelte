<script>
    import PageTitle from "../components/PageTitle.svelte";
    import {onMount} from "svelte";
    import {createRoomListStore} from "../createSearchStore";
    import RoomList from "./component/RoomList.svelte";

    let roomListStore = createRoomListStore();
    let rooms = [];

    onMount(async () => {
        await refreshRooms();

        roomListStore.subscribe(message => {
            if (message.rooms) {
                let sortedRooms = message.rooms;
                sortedRooms.sort((a, b) => a.connected_users < b.connected_users ? 1 : -1);
                rooms = sortedRooms;
            }
        });
    });

    async function refreshRooms() {
        console.debug("Fetching room list");
        let response = await fetch(`http://localhost:3030/rooms`)
        if (response.ok) {
            console.debug("Refresh request sent to Vessel server")
        } else {
            console.error("HTTP-Error: " + response.status);
        }
    }
</script>

<PageTitle title="Chat Room"/>
<RoomList {rooms}/>
<style>

</style>