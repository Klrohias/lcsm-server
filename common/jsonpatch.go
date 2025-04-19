package common

import (
	"encoding/json"

	jsonpatch "github.com/evanphx/json-patch/v5"
)

func ApplyJsonPatch(originalObject any, patchDoc []byte) error {
	originalJson, err := json.Marshal(originalObject)
	if err != nil {
		return err
	}

	patch, err := jsonpatch.DecodePatch(patchDoc)
	if err != nil {
		return err
	}

	patchedJson, err := patch.Apply(originalJson)
	if err != nil {
		return err
	}

	err = json.Unmarshal(patchedJson, originalObject)
	if err != nil {
		return err
	}

	return nil
}
