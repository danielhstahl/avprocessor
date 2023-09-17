import { List, Space, Typography, InputNumber, Card, Button, Row, Col } from 'antd';
import { DeleteOutlined, PlusOutlined } from '@ant-design/icons'
import React, { useContext } from 'react';
import { Filter, FilterContext, FilterWithIndex } from '../state/filter'
import { floatFormatter, intFormatter } from '../utils/inputParsers';
import {
    Chart as ChartJS,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
} from 'chart.js';
import { Line } from 'react-chartjs-2';
import { ChartOptions } from "chart.js"

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend
);

const options: ChartOptions = {
    responsive: true,
    elements: {
        point: {
            radius: 0
        }
    },
    scales: {
        x: {
            ticks: {
                callback: function (value: string | number) {
                    return typeof value === "number" ? Number.parseInt(this.getLabelForValue(value)) : Number.parseInt(value);
                }
            }
        }
    },
    plugins: {
        legend: {
            display: false
        }

    },
};

const { Text } = Typography



const SAMPLE_RATE = 48000
const NUM_FREQUENCIES_TO_PLOT = 100
const constructFrequencies = () => {
    let frequencies = new Float32Array(NUM_FREQUENCIES_TO_PLOT);
    let nyquist = SAMPLE_RATE / 2;
    let minLog = 1;
    let maxLog = Math.log10(nyquist);

    return frequencies.map((_, i, a) => {
        let log = minLog + (i / a.length) * (maxLog - minLog);
        return Math.pow(10, log);
    })

}
const constructVisualArray = (filters: Filter[]) => {
    const ac = new AudioContext({ sampleRate: SAMPLE_RATE })
    let magResponse = new Float32Array(NUM_FREQUENCIES_TO_PLOT);
    let phaseResponse = new Float32Array(NUM_FREQUENCIES_TO_PLOT);
    const freq = constructFrequencies()
    let freqResponse = new Array(NUM_FREQUENCIES_TO_PLOT).fill(0)
    filters.forEach(filter => {
        new BiquadFilterNode(ac, {
            type: "peaking",
            Q: filter.q,
            frequency: filter.freq,
            gain: filter.gain
        }).getFrequencyResponse(
            freq,
            magResponse,
            phaseResponse
        )
        magResponse.forEach((v, i) => {
            freqResponse[i] += Math.log10(v) //convert to DB
        })
    });
    freqResponse.forEach((v, i) => {
        freqResponse[i] = v * 20 //convert to DB
    })
    return { freq: Array.from(freq), freqResponse }
}





type ActionProps = {
    filter: FilterWithIndex,
    updateFilter: (filter: FilterWithIndex) => void,
    //removeFilter: (filter: Filter) => void
}

const FreqAction = ({ filter, updateFilter }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Frequency:</Text>
        <InputNumber
            value={filter.freq}
            onChange={v => v !== null && updateFilter({ ...filter, freq: v })}
            min={0}
            max={20000}
            {...intFormatter("hz")}
        />
    </Space>
}

const GainAction = ({ filter, updateFilter }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Gain:</Text>
        <InputNumber
            value={filter.gain}
            onChange={v => v !== null && updateFilter({ ...filter, gain: v })}
            step="0.5"
            min={-10}
            max={10}
            {...floatFormatter("db")}
        />
    </Space>
}

const QAction = ({ filter, updateFilter }: ActionProps) => {
    return <Space direction="horizontal" size="middle" >
        <Text>Q:</Text>
        <InputNumber
            value={filter.q}
            step="0.2"
            onChange={v => v !== null && updateFilter({ ...filter, q: v })}
            min={0}
            max={10}
        />
    </Space>
}


const PeqChartChartJS = ({ labels, values }: { labels: number[], values: number[] }) => {
    const jsdata = {
        labels,
        datasets: [
            {
                label: 'PEQ',
                data: values,
                borderColor: 'rgb(53, 162, 235)',
                backgroundColor: 'rgba(53, 162, 235, 0.5)',
            },
        ],
    };
    // @ts-ignore
    return <Line options={options} data={jsdata} />;

}

type SpeakerFilter = Record<string, FilterWithIndex[]>
const perSpeakerFilters: (filters: FilterWithIndex[]) => SpeakerFilter = (filters: FilterWithIndex[]) => {
    return filters.reduce<SpeakerFilter>((agg, filter) => {
        return {
            ...agg,
            [filter.speaker]: agg[filter.speaker] ? [...agg[filter.speaker], filter] : [filter]
        }
    }, {})
}
const FiltersComponent: React.FC = () => {
    const { filters, updateFilter, addFilter, removeFilter } = useContext(FilterContext)

    const speakerFilters = perSpeakerFilters(filters)
    return <Space direction="vertical" size="middle" style={{ display: 'flex' }}>
        {Object.entries(speakerFilters).map(([speaker, filters]) => {
            const results = constructVisualArray(filters)
            return <Card title={speaker} bordered={false}>
                <Row style={{ minHeight: 100 }}>
                    <Col md={24} lg={13} >
                        <List
                            itemLayout="horizontal"
                            dataSource={filters}
                            renderItem={(filter: FilterWithIndex) => (
                                <List.Item><FreqAction filter={filter} updateFilter={updateFilter} />
                                    <GainAction filter={filter} updateFilter={updateFilter} />
                                    <QAction filter={filter} updateFilter={updateFilter} />
                                    <DeleteOutlined onClick={() => removeFilter(filter)} /></List.Item>
                            )}
                            footer={<Button icon={<PlusOutlined />} onClick={() => addFilter(speaker)}>Add Filter</Button>}
                        />
                    </Col>
                    <Col xs={0} md={0} lg={11} style={{ paddingLeft: "10%" }}>
                        <PeqChartChartJS labels={results.freq} values={results.freqResponse} />
                    </Col>
                </Row>

            </Card>
        })}

    </Space>
}

export default FiltersComponent;