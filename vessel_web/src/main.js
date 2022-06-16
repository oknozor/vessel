import App from './App.svelte';
import {initHashRouter} from "@bjornlu/svelte-router";
import LogsPage from './pages/LogsPage.svelte';
import DownloadsPage from './pages/DownloadsPage.svelte';
import UploadsPage from './pages/UploadsPage.svelte';
import InterestsPage from './pages/InterestsPage.svelte';
import ChatRoomPage from "./chat/ChatRoomPage.svelte";
import SearchPage from "./search/SearchPage.svelte";

initHashRouter([
	{path: '/downloads', component: DownloadsPage},
	{path: '/uploads', component: UploadsPage},
	{path: '/search', component: SearchPage},
	{path: '/interests', component: InterestsPage},
	{path: '/logs', component: LogsPage},
	{path: '/chat-rooms', component: ChatRoomPage}
]);

const app = new App({
	target: document.body,
	props: {}
});

export default app;
