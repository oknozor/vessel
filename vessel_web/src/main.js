import App from './App.svelte';
import {initHashRouter} from "@bjornlu/svelte-router";
import LogsPage from './pages/LogsPage.svelte';
import DownloadsPage from './pages/DownloadsPage.svelte';
import UploadsPage from './pages/UploadsPage.svelte';
import SearchPage from './pages/SearchPage.svelte';
import InterestsPage from './pages/InterestsPage.svelte';

initHashRouter([
	{path: '', redirect: '/logs'},
	{path: '/downloads', component: DownloadsPage},
	{path: '/uploads', component: UploadsPage},
	{path: '/search', component: SearchPage},
	{path: '/interests', component: InterestsPage},
	{path: '/logs', component: LogsPage}
]);

const app = new App({
	target: document.body,
	props: {}
});

export default app;
