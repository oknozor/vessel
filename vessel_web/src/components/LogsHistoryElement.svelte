<script>
    export let date;
    export let hour;
    export let title;
    export let body;

    console.log(date);
</script>

<li class="tl-item">
    <div class="timestamp">
        Jan 8th 2020<br> {hour}
    </div>
    <div class="item-title">{title}</div>
    <div class="item-detail">{body}</div>
</li>

<style>
    li {
        list-style: none;
        margin: auto;
        min-height: 50px;
        border-left: 1px dashed #86D6FF;
        padding: 0 0 50px 30px;
        position: relative;
    }

    li:last-child {
        border-left: 0;
    }

    li::before {
        position: absolute;
        left: -18px;
        top: -5px;
        content: " ";
        border: 8px solid rgba(255, 255, 255, 0.74);
        border-radius: 500%;
        background: #258CC7;
        height: 20px;
        width: 20px;
        transition: all 500ms ease-in-out;

    }

    li:hover::before {
        border-color: #258CC7;
        transition: all 1000ms ease-in-out;
    }

    .item-title {
    }

    .item-detail {
        color: rgba(0, 0, 0, 0.5);
        font-size: 12px;
        max-width: 50%;
    }

    .timestamp {
        color: #8D8D8D;
        position: absolute;
        width: 100px;
        left: -131px;
        text-align: right;
        font-size: 12px;
    }
</style>
