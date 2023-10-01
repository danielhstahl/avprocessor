import { DelayAction, delayReducer, DelayType } from './delay'

describe("delayReducer", () => {
    it("sets delay type", () => {
        const results = delayReducer({ delayType: DelayType.MS }, { type: DelayAction.UPDATE, value: DelayType.FEET })
        expect(results.delayType).toEqual(DelayType.FEET)
    })
})