
import { render, screen, act } from '@testing-library/react';
import { FilterProviderComponent, FilterContext, setFiltersPure } from './filter'
import { useContext } from 'react'
import userEvent from '@testing-library/user-event'

describe("setFiltersPure", () => {
    it("filters correctly", () => {
        const results = setFiltersPure([
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 300,
                gain: 3
            },
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 800,
                gain: 3
            },
            {
                speaker: "speaker2",
                q: 0.1,
                freq: 800,
                gain: 3
            }
        ])
        expect(results).toEqual([
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 300,
                gain: 3,
                index: 1
            },
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 800,
                gain: 3,
                index: 2
            },
            {
                speaker: "speaker2",
                q: 0.1,
                freq: 800,
                gain: 3,
                index: 1
            }
        ])
    })
})

describe("FilterProviderComponent", () => {

    it("correctls adds filter", async () => {
        const user = userEvent.setup()
        const FakeComponent = () => {
            const { addFilter, filters } = useContext(FilterContext)

            return <div><button onClick={() => addFilter("myspeaker")}>click</button>{filters.map(v => <p key={v.speaker}>{v.speaker}</p>)}</div>
        }
        render(
            <FilterProviderComponent>
                <div>
                    <FakeComponent />
                </div>
            </FilterProviderComponent>
        )
        await act(async () => user.click(screen.getByRole('button', { name: /click/i })))
        const speaker = screen.getByText(/myspeaker/i);
        expect(speaker).toBeInTheDocument()
    })
    it("correctls sets filters", async () => {
        const user = userEvent.setup()
        const FakeComponent = () => {
            const { setFilters, filters } = useContext(FilterContext)

            return <div><button onClick={() => setFilters([
                {
                    speaker: "speaker1",
                    q: 0.1,
                    freq: 300,
                    gain: 3
                },
                {
                    speaker: "speaker1",
                    q: 0.1,
                    freq: 800,
                    gain: 3
                }
            ])}>click</button>{filters.map((v, i) => <p key={i}>{v.speaker + v.index}</p>)}</div>
        }
        render(
            <FilterProviderComponent>
                <div>
                    <FakeComponent />
                </div>
            </FilterProviderComponent>
        )
        await act(async () => user.click(screen.getByRole('button', { name: /click/i })))
        const speaker1 = screen.getAllByText(/speaker1/i);
        expect(speaker1.length).toEqual(2)
        const speakerIndex1 = screen.getByText(/speaker11/i);
        expect(speakerIndex1).toBeInTheDocument()
        const speakerIndex2 = screen.getByText(/speaker12/i);
        expect(speakerIndex2).toBeInTheDocument()
    })

    it("correctly updates filters", async () => {
        const user = userEvent.setup()
        const FakeComponent = () => {
            const { setFilterBase, filters } = useContext(FilterContext)

            return <div><button onClick={() => setFilterBase("5.2")}>click</button>{filters.map(v => <p key={v.speaker}>{v.speaker + v.index}</p>)}</div>
        }
        render(
            <FilterProviderComponent>
                <div>
                    <FakeComponent />
                </div>
            </FilterProviderComponent>
        )
        await act(async () => user.click(screen.getByRole('button', { name: /click/i })))
        const speaker1 = screen.getByText(/Surround Left0/i);
        expect(speaker1).toBeInTheDocument()

    })
})