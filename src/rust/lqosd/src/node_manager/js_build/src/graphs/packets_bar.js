import {DashboardGraph} from "./dashboard_graph";
import {scaleNumber} from "../helpers/scaling";

export class PacketsPerSecondBar extends DashboardGraph {
    constructor(id) {
        super(id);
        this.option = {
            xAxis: {
                type: 'value',
                axisLabel: {
                    formatter: (value) => { return scaleNumber(value, 0); }
                }
            },
            yAxis: {
                type: 'category',
                data: ['UP', 'DN'],
            },
            series: [
                {
                    type: 'bar',
                    data: [0, 0],
                }
            ]
        }
        this.option && this.chart.setOption(this.option);
    }

    update(down, up) {
        this.chart.hideLoading();
        this.option.series[0].data = [
            { value: up, itemStyle: { color: 'orange' } },
            { value: down, itemStyle: { color: 'green' } }
        ];
        this.chart.setOption(this.option);
    }
}