<script>
</script>

<div class="history-tl-container">
    <ul class="tl">
        <li class="tl-item">
            <div class="timestamp">
                3rd March 2015<br> 7:00 PM
            </div>
            <div class="item-title">Start from Shire</div>
            <div class="item-detail">Don't forget the ring</div>
        </li>
        <li class="tl-item">
            <div class="timestamp">
                19th March 2015<br> 3:00 PM
            </div>
            <div class="item-title">Kill some Orcs</div>
            <div class="item-detail">Don't enter the caves!!</div>
        </li>
        <li class="tl-item">
            <div class="timestamp">
                1st June 2015<br> 7:00 PM
            </div>
            <div class="item-title">Throw that goddamn ring in the lava</div>
            <div class="item-detail">Also, throw that Gollum too</div>
        </li>

    </ul>

</div>

<style>
    .history-tl-container {

    }

    ul {
        margin: 20px 0;
        padding: 0;
        display: inline-block;

    }

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
    }

    .timestamp {
        color: #8D8D8D;
        position: absolute;
        width: 100px;
        left: -50%;
        text-align: right;
        font-size: 12px;
    }
</style>
