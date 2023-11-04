import { DeviceAction, deviceReducer, DeviceType } from './device'

describe("deviceReducer", () => {
    it("sets device type", () => {
        const results = deviceReducer(
            { deviceType: DeviceType.MotuMk5 },
            { type: DeviceAction.UPDATE, value: DeviceType.ToppingDm7 })
        expect(results.deviceType).toEqual(DeviceType.ToppingDm7)
    })
})