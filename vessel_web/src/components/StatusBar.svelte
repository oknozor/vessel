<script>
    import StatusBarElement from "./StatusBarElement.svelte";

    export let isConnected = true;
    export let currentNumberOfConnections = 0;
    export let maxNumberOfConnections = 1024
    export let numberOfUsersDownload = 0;
    export let downloadRate = 0.0;
    export let numberOfUsersUpload = 0;
    export let uploadRate = 0.0;
</script>

<div class="z-50 fixed bottom-0 right-0 mb-7 mt-5 block pl-5 pr-5 hidden md:block">
    <button class="text-center m-0 font-medium text-sm bg-black text-white rounded-full flex flex-row leading-10 border-2 border-black border-solid transition duration-300 ease-in-out transform hover:-translate-y-1 hover:scale-105">
        {#if isConnected}
            <StatusBarElement title="Online">
<svg class="ml-4" height="100%" width="18px" focusable="false" data-prefix="fal" data-icon="wifi" role="img"
     xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512"><path
        fill="currentColor"
        d="M320 320c-44.18 0-80 35.82-80 80 0 44.19 35.83 80 80 80 44.19 0 80-35.84 80-80 0-44.18-35.82-80-80-80zm0 128c-26.47 0-48-21.53-48-48s21.53-48 48-48 48 21.53 48 48-21.53 48-48 48zm316.21-290.05C459.22-9.9 180.95-10.06 3.79 157.95c-4.94 4.69-5.08 12.51-.26 17.32l5.69 5.69c4.61 4.61 12.07 4.74 16.8.25 164.99-156.39 423.64-155.76 587.97 0 4.73 4.48 12.19 4.35 16.8-.25l5.69-5.69c4.81-4.81 4.67-12.63-.27-17.32zM526.02 270.31c-117.34-104.48-294.86-104.34-412.04 0-5.05 4.5-5.32 12.31-.65 17.2l5.53 5.79c4.46 4.67 11.82 4.96 16.66.67 105.17-93.38 264-93.21 368.98 0 4.83 4.29 12.19 4.01 16.66-.67l5.53-5.79c4.65-4.89 4.38-12.7-.67-17.2z"
></path></svg>
            </StatusBarElement>
        {:else}
            <StatusBarElement title="Offline">
            <svg height="100%" width="18" focusable="false" data-prefix="fal" data-icon="wifi-slash" role="img"
                 xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512" class="svg-inline--fa fa-wifi-slash fa-w-20 fa-3x"><path
        fill="currentColor"
        d="M324.2 320.4c-1.4-.1-2.8-.4-4.2-.4-44.2 0-80 35.8-80 80s35.8 80 80 80 80-35.8 80-80c0-8-1.5-15.5-3.7-22.8zM320 448c-26.5 0-48-21.5-48-48s21.5-48 48-48 48 21.5 48 48-21.5 48-48 48zM3.8 158c-4.9 4.7-5.1 12.5-.3 17.3l5.7 5.7c4.6 4.6 12.1 4.7 16.8.3 19.1-18.1 39.5-33.9 60.8-47.8l-26.4-20.8C40.8 126.3 21.7 141 3.8 158zM614 181.3c4.7 4.5 12.2 4.4 16.8-.3l5.7-5.7c4.8-4.8 4.7-12.6-.3-17.3C503.6 32.2 314.1.6 152.9 63.3l30.2 23.8C328.9 37.9 495.8 69.2 614 181.3zm-297.5 10.8l44.6 35.2c51.8 7.7 101.8 29.8 143.3 66.7 4.8 4.3 12.2 4 16.6-.7l5.5-5.8c4.7-4.9 4.4-12.7-.7-17.2-59.4-53-134.4-79-209.3-78.2zM637 485.2L23 1.8C19.6-1 14.5-.5 11.8 3l-10 12.5C-1 19-.4 24 3 26.7l614 483.5c3.4 2.8 8.5 2.2 11.2-1.2l10-12.5c2.8-3.5 2.3-8.5-1.2-11.3zM114 270.3c-5 4.5-5.3 12.3-.6 17.2l5.5 5.8c4.5 4.7 11.8 5 16.7.7 25.8-23 55.7-40.9 88.1-52.8L195 218.6c-29 12.7-56.4 29.8-81 51.7z"
        class=""></path></svg>
            </StatusBarElement>
        {/if}
        <StatusBarElement title="{currentNumberOfConnections}/{maxNumberOfConnections}">
            <svg height="100%" width="18" focusable="false" data-prefix="fal" data-icon="link"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><path fill="currentColor"
                                                                    d="M301.148 394.702l-79.2 79.19c-50.778 50.799-133.037 50.824-183.84 0-50.799-50.778-50.824-133.037 0-183.84l79.19-79.2a132.833 132.833 0 0 1 3.532-3.403c7.55-7.005 19.795-2.004 20.208 8.286.193 4.807.598 9.607 1.216 14.384.481 3.717-.746 7.447-3.397 10.096-16.48 16.469-75.142 75.128-75.3 75.286-36.738 36.759-36.731 96.188 0 132.94 36.759 36.738 96.188 36.731 132.94 0l79.2-79.2.36-.36c36.301-36.672 36.14-96.07-.37-132.58-8.214-8.214-17.577-14.58-27.585-19.109-4.566-2.066-7.426-6.667-7.134-11.67a62.197 62.197 0 0 1 2.826-15.259c2.103-6.601 9.531-9.961 15.919-7.28 15.073 6.324 29.187 15.62 41.435 27.868 50.688 50.689 50.679 133.17 0 183.851zm-90.296-93.554c12.248 12.248 26.362 21.544 41.435 27.868 6.388 2.68 13.816-.68 15.919-7.28a62.197 62.197 0 0 0 2.826-15.259c.292-5.003-2.569-9.604-7.134-11.67-10.008-4.528-19.371-10.894-27.585-19.109-36.51-36.51-36.671-95.908-.37-132.58l.36-.36 79.2-79.2c36.752-36.731 96.181-36.738 132.94 0 36.731 36.752 36.738 96.181 0 132.94-.157.157-58.819 58.817-75.3 75.286-2.651 2.65-3.878 6.379-3.397 10.096a163.156 163.156 0 0 1 1.216 14.384c.413 10.291 12.659 15.291 20.208 8.286a131.324 131.324 0 0 0 3.532-3.403l79.19-79.2c50.824-50.803 50.799-133.062 0-183.84-50.802-50.824-133.062-50.799-183.84 0l-79.2 79.19c-50.679 50.682-50.688 133.163 0 183.851z"
                                                                    class=""></path></svg>
        </StatusBarElement>
        <StatusBarElement title="{numberOfUsersDownload} users ({downloadRate} B/s)">
            <svg height="100%" width="18px" focusable="false" data-prefix="fal" data-icon="arrow-down"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="">
                <path fill="currentColor"
                      d="M443.5 248.5l-7.1-7.1c-4.7-4.7-12.3-4.7-17 0L241 419.9V44c0-6.6-5.4-12-12-12h-10c-6.6 0-12 5.4-12 12v375.9L28.5 241.4c-4.7-4.7-12.3-4.7-17 0l-7.1 7.1c-4.7 4.7-4.7 12.3 0 17l211 211.1c4.7 4.7 12.3 4.7 17 0l211-211.1c4.8-4.8 4.8-12.3.1-17z"
                      class=""></path></svg>
        </StatusBarElement>
        <StatusBarElement title="{numberOfUsersUpload} users ({uploadRate} B/s)">
            <svg height="100%" width="18px" focusable="false" data-prefix="fal" data-icon="arrow-up"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="">
                <path fill="currentColor"
                      d="M4.465 263.536l7.07 7.071c4.686 4.686 12.284 4.686 16.971 0L207 92.113V468c0 6.627 5.373 12 12 12h10c6.627 0 12-5.373 12-12V92.113l178.494 178.493c4.686 4.686 12.284 4.686 16.971 0l7.07-7.071c4.686-4.686 4.686-12.284 0-16.97l-211.05-211.05c-4.686-4.686-12.284-4.686-16.971 0L4.465 246.566c-4.687 4.686-4.687 12.284 0 16.97z"></path>
            </svg>
        </StatusBarElement>
    </button>
</div>