import { render, screen } from '@testing-library/react';
import { VersionProviderComponent, VersionContext } from './version'
import { useContext } from 'react'
import userEvent from '@testing-library/user-event'


describe("VersionProviderComponent", () => {

    it("correctls adds version", async () => {
        const user = userEvent.setup()
        const FakeComponent = () => {
            const { versions, addVersion } = useContext(VersionContext)

            return <div><button onClick={() => addVersion("0.1.0")}>click</button>{versions.map(v => <p key={v.version}>{v.version}</p>)}</div>
        }
        render(
            <VersionProviderComponent>
                <div>
                    <FakeComponent />
                </div>
            </VersionProviderComponent>
        )
        await user.click(screen.getByRole('button', { name: /click/i }))
        const version = screen.getByText(/0.1.0/i);
        expect(version).toBeInTheDocument()

    })
})