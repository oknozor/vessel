<script>
</script>

<div class="search-header">
    <form class="search-form" action=".">
        <div class="search-input-wrapper">
            <figure class="search-logo">
                <svg height="28" viewBox="0 0 28 28" width="28" class="css-w9q8es">
                    <path fill-rule="evenodd" clip-rule="evenodd"
                          d="M21.0819 22.6659C21.5206 23.1114 22.2316 23.1114 22.6704 22.6659C23.1091 22.2215 23.1091 21.5026 22.6704 21.0571L19.2054 17.5921C20.1841 16.2781 20.7646 14.6491 20.7646 12.8829C20.7646 8.52913 17.2355 5 12.8829 5C8.52913 5 5 8.52913 5 12.8829C5 17.2366 8.52913 20.7657 12.8829 20.7657C14.6547 20.7657 16.2916 20.1807 17.6079 19.1941L21.0819 22.6659ZM7.25225 12.8829C7.25225 9.77338 9.77225 7.25225 12.8829 7.25225C15.9924 7.25225 18.5124 9.77225 18.5124 12.8829C18.5124 15.9924 15.9924 18.5135 12.884 18.5135C9.7745 18.5135 7.25338 15.9935 7.25338 12.8829H7.25225Z"></path>
                </svg>
            </figure>
            <input placeholder="Search for music, or something else..." type="search" class="search-input" value="">
        </div>
    </form>
    <button class="wishlist-btn">Wishlist</button>
</div>

<style>
    .search-header {
        width: 100%;
        padding-top: 32px;
        padding-bottom: 32px;
        z-index: 79;
        position: sticky;
        top: 0;
        -moz-box-align: center;
        align-items: center;
        display: flex;
    }

    .search-form {
        position: relative;
        flex: 1 1 0;
    }

    .search-input-wrapper {
        position: relative;
    }

    .search-logo {
        position: absolute;
        top: -15px;
        left: -30px;
        width: 44px;
        height: 44px;
        display: flex;
        -moz-box-align: center;
        align-items: center;
        -moz-box-pack: center;
        justify-content: center;
        color: #181922;
        pointer-events: none;
    }

    .search-input {
        box-sizing: border-box;
        margin: 0;
        min-width: 0;
        display: block;
        width: 100%;
        appearance: none;
        border: medium none;
        background-color: #E8E8E9;
        color: #181922;
        height: 44px;
        line-height: 44px;
        padding: 0 44px 0 58px;
        outline: currentcolor none medium;
        border-radius: 99px;
        transition: all 300ms cubic-bezier(0.23, 1, 0.32, 1) 0s;
        pointer-events: all;
        font-size: 14px;
    }

    .wishlist-btn {
        box-sizing: border-box;
        margin: 0 0 0 8px;
        min-width: 0;
        appearance: none;
        display: inline-block;
        text-align: center;
        text-decoration: none;
        border: 0 none;
        background-color: transparent;
        color: #131314;
        box-shadow: rgba(19, 19, 20, 0.22) 0 0 0 1px, rgba(0, 0, 0, 0.25) 0 4px 4px -5px;
        cursor: pointer;
        font-weight: 500;
        outline: currentcolor none medium;
        transition: all 300ms cubic-bezier(0.23, 1, 0.32, 1) 0s;
        padding: 0 24px;
        pointer-events: all;
        font-size: 14px;
        border-radius: 99px;
        height: 42px;
        line-height: 42px;
        position: relative;
    }
</style>