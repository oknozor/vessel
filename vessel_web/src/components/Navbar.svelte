<script>
    import NavbarCategoryTitle from "./NavbarCategoryTitle.svelte";
    import NavbarLink from "./NavbarLink.svelte";
</script>

<aside class="bg-white w-64 pl-4 pr-4 pt-4 pb-24 border-solid border-r fixed inset-0 overflow-auto">
    <div class="pl-4 mb-8 mt-8">
        <div class="flex items-center font-extrabold">
            Vessel
        </div>
    </div>
    <nav class="flex flex-col items-start">
       <NavbarLink title="Downloads" to="/downloads">
                <svg height="18" focusable="false" data-prefix="fal" data-icon="arrow-down"
                     role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                     class="svg-inline--fa fa-arrow-down fa-w-14 fa-3x">
                    <path fill="currentColor"
                          d="M443.5 248.5l-7.1-7.1c-4.7-4.7-12.3-4.7-17 0L241 419.9V44c0-6.6-5.4-12-12-12h-10c-6.6 0-12 5.4-12 12v375.9L28.5 241.4c-4.7-4.7-12.3-4.7-17 0l-7.1 7.1c-4.7 4.7-4.7 12.3 0 17l211 211.1c4.7 4.7 12.3 4.7 17 0l211-211.1c4.8-4.8 4.8-12.3.1-17z"
                          class=""></path>
                </svg>
       </NavbarLink>
        <NavbarLink title="Uploads" to="/uploads">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="arrow-up"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="svg-inline--fa fa-arrow-up fa-w-14 fa-3x">
                <path fill="currentColor"
                      d="M4.465 263.536l7.07 7.071c4.686 4.686 12.284 4.686 16.971 0L207 92.113V468c0 6.627 5.373 12 12 12h10c6.627 0 12-5.373 12-12V92.113l178.494 178.493c4.686 4.686 12.284 4.686 16.971 0l7.07-7.071c4.686-4.686 4.686-12.284 0-16.97l-211.05-211.05c-4.686-4.686-12.284-4.686-16.971 0L4.465 246.566c-4.687 4.686-4.687 12.284 0 16.97z"
                      class=""></path>
            </svg>
        </NavbarLink>
        <NavbarLink title="Search" to="/search">
            <svg height="28" viewBox="0 0 28 28" width="28" class="css-w9q8es">
                <path clip-rule="evenodd"
                      d="m8.5 13.2194c0-2.6065 2.1129-4.7194 4.7194-4.7194s4.7194 2.1129 4.7194 4.7194c0 1.2918-.519 2.4625-1.36 3.3147-.0078.0072-.0155.0145-.023.0221-.0076.0075-.0149.0152-.0221.023-.8522.8407-2.0226 1.3596-3.3143 1.3596-2.6065 0-4.7194-2.1129-4.7194-4.7194zm8.5547 4.8964c-1.0566.8288-2.3882 1.323-3.8353 1.323-3.43488 0-6.2194-2.7845-6.2194-6.2194 0-3.43488 2.78452-6.2194 6.2194-6.2194 3.4349 0 6.2194 2.78452 6.2194 6.2194 0 1.4473-.4943 2.7791-1.3234 3.8358l2.6655 2.6655c.2929.2929.2929.7677 0 1.0606s-.7678.2929-1.0607 0z"
                      fill-rule="evenodd"></path>
            </svg>
        </NavbarLink>
        <NavbarLink title="Interests" to="/interests">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="check" role="img"
                 xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="svg-inline--fa fa-check fa-w-14 fa-3x">
                <path fill="currentColor"
                      d="M413.505 91.951L133.49 371.966l-98.995-98.995c-4.686-4.686-12.284-4.686-16.971 0L6.211 284.284c-4.686 4.686-4.686 12.284 0 16.971l118.794 118.794c4.686 4.686 12.284 4.686 16.971 0l299.813-299.813c4.686-4.686 4.686-12.284 0-16.971l-11.314-11.314c-4.686-4.686-12.284-4.686-16.97 0z"
                      class=""></path>
            </svg>
        </NavbarLink>
    </nav>
    <NavbarCategoryTitle title="Chat"/>
    <nav>
        <NavbarLink title="Chat Rooms" to="/chat-rooms">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="comment-music"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"
                 class="svg-inline--fa fa-comment-music fa-w-16 fa-3x">
                <path fill="currentColor"
                      d="M256 32C114.64 32 .07 125.09.07 240 .07 287.59 20 331.2 53 366.3c-14.91 39.4-45.9 72.79-46.4 73.2A24 24 0 0 0 24 480c61.49 0 110-25.7 139.07-46.3A309.89 309.89 0 0 0 256 448c141.39 0 256-93.09 256-208S397.42 32 256 32zm0 384a275.44 275.44 0 0 1-83.17-12.8l-15.22-4.79-13 9.18c-23 16.32-58.49 35.32-102.58 39.61 12-15.11 29.81-40.4 40.81-69.61L90 358.91 76.24 344.3C47.37 313.7 32.06 277.59 32.06 240c0-97 100.49-176 224-176s224 79 224 176S379.52 416 256 416zm75.18-287.25L203.23 176A16 16 0 0 0 192 191.25V290a69.82 69.82 0 0 0-16-2c-26.49 0-48 14.33-48 32s21.5 32 48 32 48-14.33 48-32V211.16l96-37.52V258a69.8 69.8 0 0 0-16-2c-26.5 0-48 14.33-48 32s21.5 32 48 32 48-14.33 48-32V144a16 16 0 0 0-20.79-15.25z"
                      class=""></path>
            </svg>
        </NavbarLink>
        <NavbarLink title="Private Chat" to="/private-chat">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="user" role="img"
                 xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="svg-inline--fa fa-user fa-w-14 fa-3x">
                <path fill="currentColor"
                      d="M313.6 288c-28.7 0-42.5 16-89.6 16-47.1 0-60.8-16-89.6-16C60.2 288 0 348.2 0 422.4V464c0 26.5 21.5 48 48 48h352c26.5 0 48-21.5 48-48v-41.6c0-74.2-60.2-134.4-134.4-134.4zM416 464c0 8.8-7.2 16-16 16H48c-8.8 0-16-7.2-16-16v-41.6C32 365.9 77.9 320 134.4 320c19.6 0 39.1 16 89.6 16 50.4 0 70-16 89.6-16 56.5 0 102.4 45.9 102.4 102.4V464zM224 256c70.7 0 128-57.3 128-128S294.7 0 224 0 96 57.3 96 128s57.3 128 128 128zm0-224c52.9 0 96 43.1 96 96s-43.1 96-96 96-96-43.1-96-96 43.1-96 96-96z"
                      class=""></path>
            </svg>
        </NavbarLink>
    </nav>
    <NavbarCategoryTitle title="Users"/>
    <nav>
        <NavbarLink title="User Info" to="/user-info">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="user" role="img"
                 xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="svg-inline--fa fa-user fa-w-14 fa-3x">
                <path fill="currentColor"
                      d="M313.6 288c-28.7 0-42.5 16-89.6 16-47.1 0-60.8-16-89.6-16C60.2 288 0 348.2 0 422.4V464c0 26.5 21.5 48 48 48h352c26.5 0 48-21.5 48-48v-41.6c0-74.2-60.2-134.4-134.4-134.4zM416 464c0 8.8-7.2 16-16 16H48c-8.8 0-16-7.2-16-16v-41.6C32 365.9 77.9 320 134.4 320c19.6 0 39.1 16 89.6 16 50.4 0 70-16 89.6-16 56.5 0 102.4 45.9 102.4 102.4V464zM224 256c70.7 0 128-57.3 128-128S294.7 0 224 0 96 57.3 96 128s57.3 128 128 128zm0-224c52.9 0 96 43.1 96 96s-43.1 96-96 96-96-43.1-96-96 43.1-96 96-96z"
                      class=""></path>
            </svg>
        </NavbarLink>
        <NavbarLink title="User Browse" to="/user-browse">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="folder"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"
                 class="svg-inline--fa fa-folder fa-w-16 fa-3x">
                <path fill="currentColor"
                      d="M194.74 96l54.63 54.63c6 6 14.14 9.37 22.63 9.37h192c8.84 0 16 7.16 16 16v224c0 8.84-7.16 16-16 16H48c-8.84 0-16-7.16-16-16V112c0-8.84 7.16-16 16-16h146.74M48 64C21.49 64 0 85.49 0 112v288c0 26.51 21.49 48 48 48h416c26.51 0 48-21.49 48-48V176c0-26.51-21.49-48-48-48H272l-54.63-54.63c-6-6-14.14-9.37-22.63-9.37H48z"
                      class=""></path>
            </svg>
        </NavbarLink>
        <NavbarLink title="Buddy List" to="/buddy-list">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="user-friends"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512"
                 class="svg-inline--fa fa-user-friends fa-w-20 fa-3x">
                <path fill="currentColor"
                      d="M480 256c53 0 96-43 96-96s-43-96-96-96-96 43-96 96 43 96 96 96zm0-160c35.3 0 64 28.7 64 64s-28.7 64-64 64-64-28.7-64-64 28.7-64 64-64zM192 256c61.9 0 112-50.1 112-112S253.9 32 192 32 80 82.1 80 144s50.1 112 112 112zm0-192c44.1 0 80 35.9 80 80s-35.9 80-80 80-80-35.9-80-80 35.9-80 80-80zm80.1 212c-33.4 0-41.7 12-80.1 12-38.4 0-46.7-12-80.1-12-36.3 0-71.6 16.2-92.3 46.9C7.2 341.3 0 363.4 0 387.2V432c0 26.5 21.5 48 48 48h288c26.5 0 48-21.5 48-48v-44.8c0-23.8-7.2-45.9-19.6-64.3-20.7-30.7-56-46.9-92.3-46.9zM352 432c0 8.8-7.2 16-16 16H48c-8.8 0-16-7.2-16-16v-44.8c0-16.6 4.9-32.7 14.1-46.4 13.8-20.5 38.4-32.8 65.7-32.8 27.4 0 37.2 12 80.2 12s52.8-12 80.1-12c27.3 0 51.9 12.3 65.7 32.8 9.2 13.7 14.1 29.8 14.1 46.4V432zm271.7-114.9C606.4 291.5 577 278 546.8 278c-27.8 0-34.8 10-66.8 10s-39-10-66.8-10c-13.2 0-26.1 3-38.1 8.1 15.2 15.4 18.5 23.6 20.2 26.6 5.7-1.6 11.6-2.6 17.9-2.6 21.8 0 30 10 66.8 10s45-10 66.8-10c21 0 39.8 9.3 50.4 25 7.1 10.5 10.9 22.9 10.9 35.7V408c0 4.4-3.6 8-8 8H416c0 17.7.3 22.5-1.6 32H600c22.1 0 40-17.9 40-40v-37.3c0-19.9-6-38.3-16.3-53.6z"
                      class=""></path>
            </svg>
        </NavbarLink>
    </nav>
    <NavbarCategoryTitle title="Server"/>
    <nav>
        <NavbarLink title="Settings" to="/settings">
            <svg height="18" focusable="false" data-prefix="fal" data-icon="sliders-v"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="svg-inline--fa fa-sliders-v fa-w-14 fa-3x">
                <path fill="currentColor"
                      d="M160 168v-48c0-13.3-10.7-24-24-24H96V8c0-4.4-3.6-8-8-8H72c-4.4 0-8 3.6-8 8v88H24c-13.3 0-24 10.7-24 24v48c0 13.3 10.7 24 24 24h40v312c0 4.4 3.6 8 8 8h16c4.4 0 8-3.6 8-8V192h40c13.3 0 24-10.7 24-24zm-32-8H32v-32h96v32zm152 160h-40V8c0-4.4-3.6-8-8-8h-16c-4.4 0-8 3.6-8 8v312h-40c-13.3 0-24 10.7-24 24v48c0 13.3 10.7 24 24 24h40v88c0 4.4 3.6 8 8 8h16c4.4 0 8-3.6 8-8v-88h40c13.3 0 24-10.7 24-24v-48c0-13.3-10.7-24-24-24zm-8 64h-96v-32h96v32zm152-224h-40V8c0-4.4-3.6-8-8-8h-16c-4.4 0-8 3.6-8 8v152h-40c-13.3 0-24 10.7-24 24v48c0 13.3 10.7 24 24 24h40v248c0 4.4 3.6 8 8 8h16c4.4 0 8-3.6 8-8V256h40c13.3 0 24-10.7 24-24v-48c0-13.3-10.7-24-24-24zm-8 64h-96v-32h96v32z"
                      class=""></path>
            </svg>
        </NavbarLink>
        <NavbarLink title="Logs" to="/logs">
            <svg height="18" focusable="false" data-prefix="fas" data-icon="align-justify"
                 role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"
                 class="svg-inline--fa fa-align-justify fa-w-14 fa-3x">
                <path fill="currentColor"
                      d="M432 416H16a16 16 0 0 0-16 16v32a16 16 0 0 0 16 16h416a16 16 0 0 0 16-16v-32a16 16 0 0 0-16-16zm0-128H16a16 16 0 0 0-16 16v32a16 16 0 0 0 16 16h416a16 16 0 0 0 16-16v-32a16 16 0 0 0-16-16zm0-128H16a16 16 0 0 0-16 16v32a16 16 0 0 0 16 16h416a16 16 0 0 0 16-16v-32a16 16 0 0 0-16-16zm0-128H16A16 16 0 0 0 0 48v32a16 16 0 0 0 16 16h416a16 16 0 0 0 16-16V48a16 16 0 0 0-16-16z"
                      class=""></path>
            </svg>
        </NavbarLink>
    </nav>
</aside>