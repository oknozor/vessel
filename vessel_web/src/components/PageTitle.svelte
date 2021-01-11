<script>
    export let title;
</script>
<header>
    <h1>{title}</h1>
</header>

<style>
    header {
        display: flex;
        flex-direction: column;
        -moz-box-align: center;
        align-items: center;
        padding-bottom: 48px;
        padding-top: 48px;
    }

    h1 {
        box-sizing: border-box;
        margin: 0;
        min-width: 0;
        color: #181922;
        font-size: 24px;
        font-weight: 400;
        line-height: 1.3;
        letter-spacing: -0.7px;
        font-family: "Inter", Helvetica, Arial, sans-serif;
        padding: 0;
    }
</style>