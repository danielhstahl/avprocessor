import userEvent from '@testing-library/user-event'
import { ROOT_ID } from '../utils/constants'
import { render, screen, waitFor, act } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { SpeakerProvider } from '../state/speaker'
import { FilterProvider } from '../state/filter'
import { VersionProvider } from '../state/version'
import Speaker from './Speakers'
import { DelayType } from '../state/delay';
describe("SpeakerComponent", () => {
    test('renders configuration version', async () => {
        const router = createMemoryRouter([
            {

                path: "/",
                element: <Speaker />,
                id: ROOT_ID,
                errorElement: <p>Uh oh, 404</p>,
                //loader: () => ({ versions: [{ version: "0.1", appliedVersion: true }], speakers: undefined, filters: undefined, appliedVersion: undefined }),
                //children: [{ path: "/", element: <div>Hello</div> }]
            },

        ], { initialEntries: ["/"] });
        render(<SpeakerProvider>
            <FilterProvider>
                <VersionProvider>
                    <RouterProvider router={router} />
                </VersionProvider>
            </FilterProvider>
        </SpeakerProvider>)
        await waitFor(() => expect(screen.getByText(/Select Configuration Version/i)).toBeInTheDocument())

    });
    test('correct version displays and update made', async () => {
        const spy = jest.fn((_: number) => Promise.resolve({ speakers: [], filters: [], selectedDistance: DelayType.FEET }))
        const router = createMemoryRouter([
            {

                path: "/",
                element: <Speaker getConfigurationProp={spy} />,
                id: ROOT_ID,
                errorElement: <p>Uh oh, 404</p>,
                //loader: () => ({ versions: [{ version: "0.1", appliedVersion: true }, { version: "0.2", appliedVersion: true }], speakers: undefined, filters: undefined, appliedVersion: undefined }),
                //children: [{ path: "/", element: <div>Hello</div> }]
            },

        ], { initialEntries: ["/"] });

        render(<SpeakerProvider>
            <FilterProvider>
                <VersionProvider versionState={{
                    versions: [
                        { version: 1, appliedVersion: true, versionDate: "2023" },
                        { version: 2, appliedVersion: true, versionDate: "2023" }
                    ]
                }}>
                    <RouterProvider router={router} />
                </VersionProvider>
            </FilterProvider>
        </SpeakerProvider >)

        const select = await waitFor(() => screen.getAllByRole('combobox').at(0))
        if (select) {
            await act(async () => await userEvent.click(select))
        }
        const initSelect = await waitFor(() => screen.getByTitle('1 (2023)'))
        expect(initSelect.className).toContain("ant-select-item-option-active")

        await waitFor(() => screen.getByTitle('2 (2023)'))
        await act(async () => await userEvent.click(screen.getByTitle('2 (2023)')))
        const secondSelect = await waitFor(() => screen.getAllByTitle('2 (2023)').at(1))
        expect(secondSelect?.className).toContain("ant-select-item-option-active")

        expect(spy).toHaveBeenCalledWith(2)
    });
})