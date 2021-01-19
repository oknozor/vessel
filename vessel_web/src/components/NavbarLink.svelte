<script>
    import {Link} from '@bjornlu/svelte-router'
    export let title;
    export let to;
</script>

<Link class="nav-link flex items-center mb-1 w-full rounded cursor-pointer transition-all duration-150 ease-in-out hover:bg-gray-100 hover:text-gray-900" to="{to}">
    <div class="flex items-center justify-center h-10 w-9">
        <slot/>
    </div>
    {title}
</Link>

<style>
    :global(.link-active) {
        color: #181922;
        background-color: #F3F4F4;
    }

    :global(.nav-link) {
        font-size: 15px;
        color: #46474e;
    }

    :global(.nav-link:visited) {
        color: #46474E;
    }
</style>