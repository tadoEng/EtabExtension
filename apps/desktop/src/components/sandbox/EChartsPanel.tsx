import ReactECharts from 'echarts-for-react';

const stories = ['B1', 'GF', 'L1', 'L2', 'L3', 'L4', 'L5', 'L6', 'L7', 'L8'];

const driftX = [0.0, 0.12, 0.45, 0.68, 0.82, 0.91, 0.85, 0.76, 0.61, 0.38];
const driftY = [0.0, 0.10, 0.38, 0.59, 0.74, 0.88, 0.83, 0.70, 0.55, 0.32];
const shearX = [1200, 1050, 880, 720, 580, 450, 330, 220, 120, 45];
const shearY = [1100, 980, 820, 670, 540, 420, 310, 200, 110, 40];

const driftLimit = 0.5;

export function EChartsPanel() {
    const driftOption = {
        backgroundColor: 'transparent',
        title: {
            text: 'Story Drift Ratio (%)',
            textStyle: { color: '#94a3b8', fontSize: 13, fontWeight: 500 },
            left: 'center',
            top: 8,
        },
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        legend: {
            data: ['X direction', 'Y direction', 'Limit'],
            bottom: 8,
            textStyle: { color: '#94a3b8', fontSize: 11 },
        },
        grid: { left: 60, right: 20, top: 50, bottom: 50 },
        xAxis: {
            type: 'value',
            axisLabel: { color: '#64748b', fontSize: 11 },
            splitLine: { lineStyle: { color: '#1e293b' } },
        },
        yAxis: {
            type: 'category',
            data: stories,
            axisLabel: { color: '#94a3b8', fontSize: 11 },
            splitLine: { lineStyle: { color: '#1e293b' } },
        },
        series: [
            {
                name: 'X direction',
                type: 'bar',
                data: driftX,
                itemStyle: { color: '#3b82f6' },
                barGap: '10%',
            },
            {
                name: 'Y direction',
                type: 'bar',
                data: driftY,
                itemStyle: { color: '#06b6d4' },
            },
            {
                name: 'Limit',
                type: 'line',
                data: stories.map(() => driftLimit),
                lineStyle: { color: '#ef4444', type: 'dashed', width: 1.5 },
                itemStyle: { color: '#ef4444' },
                symbol: 'none',
            },
        ],
    };

    const shearOption = {
        backgroundColor: 'transparent',
        title: {
            text: 'Story Shear (kN)',
            textStyle: { color: '#94a3b8', fontSize: 13, fontWeight: 500 },
            left: 'center',
            top: 8,
        },
        tooltip: { trigger: 'axis' },
        legend: {
            data: ['X direction', 'Y direction'],
            bottom: 8,
            textStyle: { color: '#94a3b8', fontSize: 11 },
        },
        grid: { left: 60, right: 20, top: 50, bottom: 50 },
        xAxis: {
            type: 'value',
            axisLabel: { color: '#64748b', fontSize: 11 },
            splitLine: { lineStyle: { color: '#1e293b' } },
        },
        yAxis: {
            type: 'category',
            data: stories,
            axisLabel: { color: '#94a3b8', fontSize: 11 },
            splitLine: { lineStyle: { color: '#1e293b' } },
        },
        series: [
            {
                name: 'X direction',
                type: 'line',
                data: shearX,
                smooth: true,
                lineStyle: { color: '#8b5cf6', width: 2 },
                itemStyle: { color: '#8b5cf6' },
                areaStyle: { color: 'rgba(139,92,246,0.1)' },
            },
            {
                name: 'Y direction',
                type: 'line',
                data: shearY,
                smooth: true,
                lineStyle: { color: '#f59e0b', width: 2 },
                itemStyle: { color: '#f59e0b' },
                areaStyle: { color: 'rgba(245,158,11,0.1)' },
            },
        ],
    };

    return (
        <div className="w-full h-full grid grid-rows-2 gap-4">
            <div className="bg-slate-900 rounded-lg border border-border/40 overflow-hidden">
                <ReactECharts
                    option={driftOption}
                    style={{ width: '100%', height: '100%' }}
                    theme="dark"
                />
            </div>
            <div className="bg-slate-900 rounded-lg border border-border/40 overflow-hidden">
                <ReactECharts
                    option={shearOption}
                    style={{ width: '100%', height: '100%' }}
                    theme="dark"
                />
            </div>
        </div>
    );
}
